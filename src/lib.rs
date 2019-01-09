//! A library for handling [International Standard Book Number], or ISBNs.
//!
//! # Examples
//!
//! ```
//! use isbn::{Isbn10, Isbn13};
//!
//! let isbn_10 = Isbn10::new(8, 9, 6, 6, 2, 6, 1, 2, 6, 4);
//! assert_eq!(isbn_10.hyphenate(), Ok("89-6626-126-4".to_string()));
//! assert_eq!(isbn_10.agency(), Ok("Korea, Republic".to_string()));
//! assert_eq!("89-6626-126-4".parse(), Ok(isbn_10));
//!
//! let isbn_13 = Isbn13::new(9, 7, 8, 1, 4, 9, 2, 0, 6, 7, 6, 6, 5);
//! assert_eq!(isbn_13.hyphenate(), Ok("978-1-4920-6766-5".to_string()));
//! assert_eq!(isbn_13.agency(), Ok("English language".to_string()));
//! assert_eq!("978-1-4920-6766-5".parse(), Ok(isbn_13));
//! ```
//!
//! [International Standard Book Number]: https://www.isbn-international.org/

use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

/// An International Standard Book Number, either ISBN10 or ISBN13.
///
/// # Examples
///
/// ```
/// use isbn::{Isbn, Isbn10, Isbn13};
///
/// let isbn_10 = Isbn::_10(Isbn10::new(8, 9, 6, 6, 2, 6, 1, 2, 6, 4));
/// let isbn_13 = Isbn::_13(Isbn13::new(9, 7, 8, 1, 4, 9, 2, 0, 6, 7, 6, 6, 5));
///
/// assert_eq!("89-6626-126-4".parse(), Ok(isbn_10));
/// assert_eq!("978-1-4920-6766-5".parse(), Ok(isbn_13));
/// ```
#[derive(Debug, PartialEq)]
pub enum Isbn {
    _10(Isbn10),
    _13(Isbn13),
}

struct Group {
    agency: String,
    segment_length: usize,
}

impl Isbn {
    /// Returns `true` if this is a valid ISBN code.
    pub fn is_valid(&self) -> bool {
        match *self {
            Isbn::_10(ref c) => c.is_valid(),
            Isbn::_13(ref c) => c.is_valid(),
        }
    }

    pub fn hyphenate(&self) -> Result<String, IsbnError> {
        match *self {
            Isbn::_10(ref c) => c.hyphenate(),
            Isbn::_13(ref c) => c.hyphenate(),
        }
    }

    pub fn agency(&self) -> Result<String, IsbnError> {
        match *self {
            Isbn::_10(ref c) => c.agency(),
            Isbn::_13(ref c) => c.agency(),
        }
    }
}

impl fmt::Display for Isbn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Isbn::_10(ref c) => c.fmt(f),
            Isbn::_13(ref c) => c.fmt(f),
        }
    }
}

impl From<Isbn10> for Isbn {
    fn from(isbn10: Isbn10) -> Isbn {
        Isbn::_10(isbn10)
    }
}

impl From<Isbn13> for Isbn {
    fn from(isbn13: Isbn13) -> Isbn {
        Isbn::_13(isbn13)
    }
}

impl FromStr for Isbn {
    type Err = IsbnError;
    fn from_str(s: &str) -> Result<Isbn, IsbnError> {
        Parser::new(s).read_isbn()
    }
}

/// 10-digit ISBN format.
#[derive(Debug, PartialEq)]
pub struct Isbn10 {
    digits: [u8; 10],
}

impl Isbn10 {
    /// Creates a new ISBN10 code from 10 digits.
    ///
    /// # Examples
    ///
    /// ```
    /// use isbn::Isbn10;
    ///
    /// let isbn10 = Isbn10::new(8, 9, 6, 6, 2, 6, 1, 2, 6, 4);
    /// ```
    pub fn new(a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8, h: u8, i: u8, j: u8) -> Isbn10 {
        Isbn10 {
            digits: [a, b, c, d, e, f, g, h, i, j],
        }
    }

    fn calculate_check_digit(digits: &[u8]) -> u8 {
        let sum: usize = digits
            .iter()
            .enumerate()
            .take(9)
            .map(|(i, &d)| d as usize * (10 - i))
            .sum();
        let check_digit = (11 - (sum % 11)) % 11;
        check_digit as u8
    }

    /// Returns `true` if this is a valid ISBN10 code.
    pub fn is_valid(&self) -> bool {
        Isbn10::calculate_check_digit(&self.digits) == *self.digits.last().unwrap()
    }

    fn registration_group(
        &self,
        registration_group_segment_legnth: usize,
    ) -> Result<Group, IsbnError> {
        Isbn::get_registration_group(
            &self.group_prefix(registration_group_segment_legnth),
            self.segment(registration_group_segment_legnth),
        )
    }

    pub fn hyphenate(&self) -> Result<String, IsbnError> {
        let registration_group_segment_legnth =
            Isbn::get_ean_ucc_group("978", self.segment(0))?.segment_length;
        let registrant_segment_length = self
            .registration_group(registration_group_segment_legnth)?
            .segment_length;
        let hyphen_at = [
            registration_group_segment_legnth,
            registration_group_segment_legnth + registrant_segment_length,
            9,
        ];

        let mut hyphenated = String::new();
        for (i, character) in self.to_string().chars().enumerate() {
            if hyphen_at.contains(&i) {
                hyphenated.push('-')
            }
            hyphenated.push(character);
        }
        Ok(hyphenated)
    }

    pub fn agency(&self) -> Result<String, IsbnError> {
        let registration_group_segment_legnth =
            Isbn::get_ean_ucc_group("978", self.segment(0))?.segment_length;

        Ok(self
            .registration_group(registration_group_segment_legnth)?
            .agency)
    }

    fn segment(&self, base: usize) -> u32 {
        let s = format!(
            "{}{}{}{}{}{}{}",
            self.digits.get(base).unwrap_or(&0),
            self.digits.get(base + 1).unwrap_or(&0),
            self.digits.get(base + 2).unwrap_or(&0),
            self.digits.get(base + 3).unwrap_or(&0),
            self.digits.get(base + 4).unwrap_or(&0),
            self.digits.get(base + 5).unwrap_or(&0),
            self.digits.get(base + 6).unwrap_or(&0),
        );
        s.parse().unwrap()
    }

    fn group_prefix(&self, length: usize) -> String {
        [
            "978",
            &self.digits[..length]
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(""),
        ]
        .join("-")
    }
}

impl fmt::Display for Isbn10 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sum = self.digits.iter().fold(String::new(), |acc, &d| match d {
            10 => acc + "X",
            _ => acc + &d.to_string(),
        });
        write!(f, "{}", sum)
    }
}

impl FromStr for Isbn10 {
    type Err = IsbnError;
    fn from_str(s: &str) -> Result<Isbn10, IsbnError> {
        Parser::new(s).read_isbn10()
    }
}

/// 13-digit ISBN format.
#[derive(Debug, PartialEq)]
pub struct Isbn13 {
    digits: [u8; 13],
}

impl Isbn13 {
    /// Creates a new ISBN13 code from 13 digits.
    ///
    /// # Examples
    ///
    /// ```
    /// use isbn::Isbn13;
    ///
    /// let isbn13 = Isbn13::new(9, 7, 8, 1, 4, 9, 2, 0, 6, 7, 6, 6, 5);
    /// ```
    pub fn new(
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        g: u8,
        h: u8,
        i: u8,
        j: u8,
        k: u8,
        l: u8,
        m: u8,
    ) -> Isbn13 {
        Isbn13 {
            digits: [a, b, c, d, e, f, g, h, i, j, k, l, m],
        }
    }

    fn calculate_check_digit(digits: &[u8]) -> u8 {
        let sum: usize = digits
            .iter()
            .enumerate()
            .take(12)
            .map(|(i, &d)| d as usize * (3 - 2 * ((i + 1) % 2)))
            .sum();
        let check_digit = (10 - (sum % 10)) % 10;
        check_digit as u8
    }

    /// Returns `true` if this is a valid ISBN13 code.
    pub fn is_valid(&self) -> bool {
        Isbn13::calculate_check_digit(&self.digits) == *self.digits.last().unwrap()
    }

    fn ean_ucc_group(&self) -> Result<Group, IsbnError> {
        Isbn::get_ean_ucc_group(
            &format!("{}{}{}", self.digits[0], self.digits[1], self.digits[2]),
            self.segment(0),
        )
    }

    fn registration_group(
        &self,
        registration_group_segment_legnth: usize,
    ) -> Result<Group, IsbnError> {
        Isbn::get_registration_group(
            &self.group_prefix(registration_group_segment_legnth),
            self.segment(registration_group_segment_legnth),
        )
    }

    pub fn hyphenate(&self) -> Result<String, IsbnError> {
        let registration_group_segment_legnth = self.ean_ucc_group()?.segment_length;
        let registrant_segment_length = self
            .registration_group(registration_group_segment_legnth)?
            .segment_length;
        let hyphen_at = [
            3,
            3 + registration_group_segment_legnth,
            3 + registration_group_segment_legnth + registrant_segment_length,
            12,
        ];

        let mut hyphenated = String::new();
        for (i, character) in self.to_string().chars().enumerate() {
            if hyphen_at.contains(&i) {
                hyphenated.push('-')
            }
            hyphenated.push(character);
        }
        Ok(hyphenated)
    }

    pub fn agency(&self) -> Result<String, IsbnError> {
        let registration_group_segment_legnth = self.ean_ucc_group()?.segment_length;

        Ok(self
            .registration_group(registration_group_segment_legnth)?
            .agency)
    }

    fn segment(&self, base: usize) -> u32 {
        let s = format!(
            "{}{}{}{}{}{}{}",
            self.digits.get(base + 3).unwrap_or(&0),
            self.digits.get(base + 4).unwrap_or(&0),
            self.digits.get(base + 5).unwrap_or(&0),
            self.digits.get(base + 6).unwrap_or(&0),
            self.digits.get(base + 7).unwrap_or(&0),
            self.digits.get(base + 8).unwrap_or(&0),
            self.digits.get(base + 9).unwrap_or(&0),
        );
        s.parse().unwrap()
    }

    fn group_prefix(&self, length: usize) -> String {
        format!(
            "{}-{}",
            &self.digits[..3]
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join(""),
            &self.digits[3..(3 + length) as usize]
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("")
        )
    }
}

impl fmt::Display for Isbn13 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sum = self
            .digits
            .iter()
            .fold(String::new(), |acc, &d| acc + &d.to_string());
        write!(f, "{}", sum)
    }
}

impl From<Isbn10> for Isbn13 {
    fn from(isbn10: Isbn10) -> Isbn13 {
        let mut v = vec![9, 7, 8];
        v.extend_from_slice(&isbn10.digits[..9]);
        let c = Isbn13::calculate_check_digit(&v);
        let d = isbn10.digits;
        Isbn13::new(
            9, 7, 8, d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], c,
        )
    }
}

impl FromStr for Isbn13 {
    type Err = IsbnError;
    fn from_str(s: &str) -> Result<Isbn13, IsbnError> {
        Parser::new(s).read_isbn13()
    }
}

/// An error which can be returned when parsing an ISBN.
#[derive(Debug, PartialEq)]
pub enum IsbnError {
    /// The given string is too short or too long to be an ISBN.
    InvalidLength,
    /// Encountered an invalid digit while parsing.
    InvalidDigit,
    /// Encountered an invalid ISBN registration group.
    InvalidGroup,
    /// Encountered a range not defined for use at this time.
    UndefinedRange,
}

impl From<ParseIntError> for IsbnError {
    fn from(_: ParseIntError) -> Self {
        IsbnError::InvalidDigit
    }
}

struct Parser {
    digits: Vec<u8>,
}

impl Parser {
    pub fn new(s: &str) -> Parser {
        let digits = s
            .replace("-", "")
            .replace(" ", "")
            .chars()
            .map(|c| match c {
                'X' => 10,
                _ => c.to_digit(10).unwrap_or(0),
            } as u8)
            .collect();
        Parser { digits: digits }
    }

    fn read_isbn(&mut self) -> Result<Isbn, IsbnError> {
        match self.digits.len() {
            10 => self.read_isbn10().map(Isbn::_10),
            13 => self.read_isbn13().map(Isbn::_13),
            _ => Err(IsbnError::InvalidLength),
        }
    }

    fn read_isbn13(&mut self) -> Result<Isbn13, IsbnError> {
        let check_digit = Isbn13::calculate_check_digit(&self.digits);
        if check_digit == *self.digits.last().unwrap() {
            let d = &self.digits;
            Ok(Isbn13::new(
                d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9], d[10], d[11], d[12],
            ))
        } else {
            Err(IsbnError::InvalidDigit)
        }
    }

    fn read_isbn10(&mut self) -> Result<Isbn10, IsbnError> {
        let check_digit = Isbn10::calculate_check_digit(&self.digits);
        if check_digit == *self.digits.last().unwrap() {
            let d = &self.digits;
            Ok(Isbn10::new(
                d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9],
            ))
        } else {
            Err(IsbnError::InvalidDigit)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str_isbn10() {
        // Wikipedia ISBN-10 check digit calculation example
        assert!(Isbn::from_str("0-306-40615-2").is_ok());

        // Wikipedia ISBN-10 check digit calculation invalid example
        assert!(Isbn::from_str("99999-999-9-X").is_err());

        // Wikipedia Registrant element examples
        assert!(Isbn::from_str("99921-58-10-7").is_ok());
        assert!(Isbn::from_str("9971-5-0210-0").is_ok());
        assert!(Isbn::from_str("960-425-059-0").is_ok());
        assert!(Isbn::from_str("80-902734-1-6").is_ok());
        assert!(Isbn::from_str("85-359-0277-5").is_ok());
        assert!(Isbn::from_str("1-84356-028-3").is_ok());
        assert!(Isbn::from_str("0-684-84328-5").is_ok());
        assert!(Isbn::from_str("0-8044-2957-X").is_ok());
        assert!(Isbn::from_str("0-85131-041-9").is_ok());
        assert!(Isbn::from_str("0-943396-04-2").is_ok());
        assert!(Isbn::from_str("0-9752298-0-X").is_ok());
    }

    #[test]
    fn test_from_str_isbn13() {
        // Wikipedia Example
        assert!(Isbn13::from_str("978-3-16-148410-0").is_ok());

        // Wikipedia ISBN-13 check digit calculation example
        assert!(Isbn13::from_str("978-0-306-40615-7").is_ok());
    }
}
