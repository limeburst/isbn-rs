use std::path::Path;

use crate::{Group, Isbn, IsbnError, Isbn10, Isbn13, IsbnObject};
use std::io::BufReader;
use std::fs::File;

use quick_xml::{events::Event, Reader};
use std::str::FromStr;
use std::num::NonZeroUsize;
use arrayvec::ArrayString;

use indexmap::IndexMap;

struct Segment {
    name: String,
    // (start, stop, ?length).
    ranges: Vec<((u32, u32), Option<NonZeroUsize>)>,
}

pub struct IsbnRange {
    serial_number: Option<String>,
    date: String,
    ean_ucc_group: IndexMap<u16, Segment>,
    registration_group: IndexMap<(u16, u32), Segment>,
}

pub enum IsbnRangeError {
    NoIsbnRangeMessageTag,
    NoEanUccPrefixes,
    NoEanUccPrefix,
    NoRegistrationGroups,
    NoGroup,
    NoMessageDate,
    PrefixTooLong,
    LengthTooLarge,
    NoDashInRange,

}

fn read_xml_tag(reader: &mut Reader<BufReader<File>>, buf: &mut Vec<u8>, name: &[u8]) -> Option<String> {
    match reader.read_event(buf).ok()? {
        Event::Start(e) => if e.name() != name {
            return None;
        },
        _ => return None,
    };
    buf.clear();
    let res = match reader.read_event(buf).ok()? {
        Event::Text(e) => e.unescape_and_decode(&reader).ok()?,
        _ => return None,
    };
    match reader.read_event(buf).ok()? {
        Event::End(e) => if e.name() != name {
            return None;
        },
        _ => return None,
    };
    buf.clear();
    Some(res)
}

impl Segment {
    fn from_reader(reader: &mut Reader<BufReader<File>>, buf: &mut Vec<u8>) -> Option<Self> {
        let name = read_xml_tag(reader, buf, b"Agency")?;

        let mut ranges = Vec::new();

        match reader.read_event(buf).ok()? {
            Event::Start(e) => if e.name() != b"Rules" {
                return None;
            },
            _ => return None,
        };
        buf.clear();

        loop {
            match reader.read_event(buf).ok()? {
                Event::Start(e) => if e.name() != b"Rule" {
                    return None;
                },
                Event::End(e) => if e.name() == b"Rules" {
                    break;
                }
                _ => return None,
            };
            buf.clear();

            let range = read_xml_tag(reader, buf, b"Range")?;
            let length = read_xml_tag(reader, buf, b"Length")?;

            ranges.push((
                {
                    let mid = range.find("-")?;
                    let (a, b) = range.split_at(mid);
                    (u32::from_str(a).ok()?, u32::from_str(b.split_at(1).1).ok()?)
                }
                ,
                {
                    assert_eq!(length.len(), 1);
                    let length = usize::from_str_radix(&length, 10).ok()?;
                    assert!(length <= 7);
                    NonZeroUsize::new(length)
                })
            );

            match reader.read_event(buf).ok()? {
                Event::End(e) => if e.name() != b"Rule" {
                    return None;
                },
                _ => return None,
            };
            buf.clear();

        }

        match reader.read_event(buf).ok()? {
            Event::End(e) => match e.name() {
                b"EAN.UCC" | b"Group" => {},
                _ => return None,
            },
            _ => return None,
        };
        buf.clear();

        Some(Segment {
            name,
            ranges,
        })
    }

    fn group(&self, segment: u32) -> Result<Group, IsbnError> {
        for ((start, stop), length) in &self.ranges {
            if segment >= *start && segment < *stop {
                let segment_length = usize::from(length.ok_or(IsbnError::UndefinedRange)?);
                return Ok(Group {
                    name: &self.name,
                    segment_length
                })
            }
        }
        Err(IsbnError::InvalidGroup)
    }
}
impl IsbnRange {
    fn read_ean_ucc_group(reader: &mut Reader<BufReader<File>>, buf: &mut Vec<u8>) -> Option<IndexMap<u16, Segment>> {
        buf.clear();
        let mut res = IndexMap::new();
        loop {
            match reader.read_event(buf).ok()? {
                Event::Start(e) => if e.name() != b"EAN.UCC" { return None; },
                Event::End(e) if e.name() == b"EAN.UCCPrefixes" => {
                    return Some(res);
                }
                _ => return None,
            };
            buf.clear();

            let mut prefix_val = 0u16;
            for (i, char) in read_xml_tag(reader, buf, b"Prefix")?.chars().enumerate() {
                assert!(i < 3);
                prefix_val = (prefix_val << 4) | char.to_digit(10)? as u16;
            };

            res.insert(prefix_val, Segment::from_reader(reader, buf)?);
        }
    }

    fn read_registration_group(reader: &mut Reader<BufReader<File>>, buf: &mut Vec<u8>) -> Option<IndexMap<(u16, u32), Segment>> {
        buf.clear();
        let mut res = IndexMap::new();
        loop {
            match reader.read_event(buf).ok()? {
                Event::Start(e) => if e.name() != b"Group" { return None; },
                Event::End(e) if e.name() == b"RegistrationGroups" => {
                    return Some(res);
                }
                _ => return None,
            };
            buf.clear();

            let mut prefix_val = 0u16;
            let mut registration_group_element = 0u32;
            for (i, char) in read_xml_tag(reader, buf, b"Prefix")?.chars().enumerate() {
                if i < 3 {
                    prefix_val = (prefix_val << 4) | char.to_digit(10)? as u16;
                } else if i >= 4 {
                    registration_group_element = (registration_group_element << 4) | char.to_digit(10)?;
                }
            };

            res.insert((prefix_val, registration_group_element), Segment::from_reader(reader, buf)?);
        }
    }

    pub fn from_file<P: AsRef<Path>>(p: P) -> Option<Self> {
        let mut reader = Reader::from_reader(BufReader::new(File::open(p).ok()?));
        reader.trim_text(true);
        let mut buf = Vec::new();
        loop {
            match reader.read_event(&mut buf).ok()? {
                Event::Start(e) => if e.name() == b"ISBNRangeMessage" {
                    break;
                }
                _ => {}
            }
            buf.clear();
        }

        let _ = read_xml_tag(&mut reader, &mut buf, b"MessageSource");
        let serial_number = read_xml_tag(&mut reader, &mut buf, b"MessageSerialNumber");
        let date = read_xml_tag(&mut reader, &mut buf, b"MessageDate")?;

        match reader.read_event(&mut buf).ok()? {
            Event::Start(e) => if e.name() != b"EAN.UCCPrefixes" {
                return None;
            }
            _ => {}
        }
        buf.clear();
        let ean_ucc_group = Self::read_ean_ucc_group(&mut reader, &mut buf)?;
        match reader.read_event(&mut buf).ok()? {
            Event::Start(e) => if e.name() != b"RegistrationGroups" {
                return None;
            }
            _ => {}
        }

        buf.clear();
        let registration_group = Self::read_registration_group(&mut reader, &mut buf)?;
        Some(IsbnRange {
            serial_number,
            date,
            ean_ucc_group,
            registration_group,
        })
    }

    pub fn hyphenate_isbn(&self, isbn: &Isbn) -> Result<ArrayString<[u8; 17]>, IsbnError> {
        match isbn {
            Isbn::_10(isbn) => self.hyphenate_isbn_object(isbn),
            Isbn::_13(isbn) => self.hyphenate_isbn_object(isbn),
        }
    }

    pub fn hyphenate_isbn10(&self, isbn: &Isbn10) -> Result<ArrayString<[u8; 17]>, IsbnError> {
        self.hyphenate_isbn_object(isbn)
    }

    pub fn hyphenate_isbn13(&self, isbn: &Isbn13) -> Result<ArrayString<[u8; 17]>, IsbnError> {
        self.hyphenate_isbn_object(isbn)
    }

    fn hyphenate_isbn_object(&self, isbn: &impl IsbnObject) -> Result<ArrayString<[u8; 17]>, IsbnError> {
        let segment = self.ean_ucc_group.get(&isbn.prefix_element()).ok_or(IsbnError::InvalidGroup)?;
        let registration_group_segment_length = segment.group(isbn.segment(0))?.segment_length;
        let segment = self.registration_group.get(&(isbn.prefix_element(), isbn.group_prefix(registration_group_segment_length))).ok_or(IsbnError::InvalidGroup)?;
        let registrant_segment_length = segment.group(isbn.segment(registration_group_segment_length))?.segment_length;

        let hyphen_at = [
            registration_group_segment_length,
            registration_group_segment_length + registrant_segment_length,
        ];

        Ok(isbn.hyphenate_with(hyphen_at))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_isbn_range_opens() {
        assert!(IsbnRange::from_file("./isbn-ranges/RangeMessage.xml").is_some());
    }

    #[test]
    fn test_hyphenation() {
        let range = IsbnRange::from_file("./isbn-ranges/RangeMessage.xml").unwrap();
        assert!(range.hyphenate_isbn(&Isbn::from_str("0-9752298-0-X").unwrap()).is_ok());
        assert!(range.hyphenate_isbn(&Isbn::from_str("978-3-16-148410-0").unwrap()).is_ok());
    }
}