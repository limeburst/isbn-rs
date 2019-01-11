use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use codegen::{Block, Function, Scope};
use roxmltree::{Document, Node};

struct Group {
    agency: String,
    prefix: String,
    rules: Vec<Rule>,
}

struct Rule {
    min: u32,
    max: u32,
    length: usize,
}

/// Parse Registration Group and Registrant range length rules.
fn parse_rules(group: Node) -> Vec<Rule> {
    group
        .descendants()
        .filter(|n| n.has_tag_name("Rule"))
        .map(|r| {
            let range: Vec<u32> = r
                .descendants()
                .find(|n| n.has_tag_name("Range"))
                .unwrap()
                .text()
                .unwrap()
                .split('-')
                .map(|i| i.parse().unwrap())
                .collect();

            let length = r
                .descendants()
                .find(|n| n.has_tag_name("Length"))
                .unwrap()
                .text()
                .unwrap()
                .parse()
                .unwrap();

            Rule {
                min: range[0],
                max: range[1],
                length: length,
            }
        })
        .collect()
}

/// Parse EAN.UCC and Registration Group element.
fn parse_group(group: Node) -> Group {
    let prefix = group
        .descendants()
        .find(|n| n.has_tag_name("Prefix"))
        .unwrap()
        .text()
        .unwrap()
        .to_string();

    let agency = group
        .descendants()
        .find(|n| n.has_tag_name("Agency"))
        .unwrap()
        .text()
        .unwrap()
        .to_string();

    Group {
        agency: agency,
        prefix: prefix,
        rules: parse_rules(group),
    }
}

fn codegen_find_group(name: &str, groups: Vec<Group>) -> Function {
    let mut fn_get_group = Function::new(name);
    fn_get_group.arg("prefix", "&str");
    fn_get_group.arg("segment", "u32");
    fn_get_group.ret("Result<Group<'static>, IsbnError>");

    let mut match_prefix = Block::new("match prefix");
    for group in groups {
        match_prefix.line(format!("\"{}\" =>", group.prefix));

        let mut let_length_eq_match_segment = Block::new("let length = match segment");
        for rule in &group.rules {
            let_length_eq_match_segment.line(match rule.length {
                0 => format!(
                    "{} ... {} => Err(IsbnError::UndefinedRange),",
                    rule.min, rule.max
                ),
                _ => format!("{} ... {} => Ok({}),", rule.min, rule.max, rule.length),
            });
        }
        let_length_eq_match_segment.line("_ => Err(IsbnError::InvalidGroup)");
        let_length_eq_match_segment.after(";");

        let mut ok_group = Block::new("Ok(Group");
        ok_group.line(format!("name: \"{}\",", group.agency));
        ok_group.line("segment_length: length?");
        ok_group.after(")");

        let mut segment_match_block = Block::new("");
        segment_match_block.push_block(let_length_eq_match_segment);
        segment_match_block.push_block(ok_group);

        match_prefix.push_block(segment_match_block);
    }
    match_prefix.line("_ => Err(IsbnError::InvalidGroup)");

    fn_get_group.push_block(match_prefix);
    fn_get_group
}

fn main() {
    let mut f = File::open("isbn-ranges/RangeMessage.xml").unwrap();
    let mut text = String::new();
    f.read_to_string(&mut text).unwrap();

    let range_message = Document::parse(&text).unwrap();
    let ean_ucc_groups = range_message
        .descendants()
        .filter(|d| d.tag_name().name() == "EAN.UCC")
        .map(|g| parse_group(g))
        .collect();
    let registration_groups = range_message
        .descendants()
        .filter(|d| d.tag_name().name() == "Group")
        .map(|g| parse_group(g))
        .collect();

    let mut scope = Scope::new();
    let impl_isbn = scope.new_impl("Isbn");
    impl_isbn.push_fn(codegen_find_group("get_ean_ucc_group", ean_ucc_groups));
    impl_isbn.push_fn(codegen_find_group(
        "get_registration_group",
        registration_groups,
    ));

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated.rs");

    let mut f = File::create(&dest_path).unwrap();
    f.write_all(scope.to_string().as_bytes()).unwrap();
}
