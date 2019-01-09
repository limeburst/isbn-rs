#![no_std]
//! A library for handling [International Standard Book Number], or ISBNs.
//!
//! # Examples
//!
//! ```
//! use isbn::{Isbn, Isbn10, Isbn13};
//!
//! let isbn_10 = Isbn::_10(Isbn10::new(0, 3, 4, 0, 0, 1, 3, 8, 1, 8).expect("Invalid ISBN"));
//! let isbn_13 = Isbn::_13(Isbn13::new(9, 7, 8, 0, 3, 4, 0, 0, 1, 3, 8, 1, 6).expect("Invalid ISBN"));
//!
//! assert_eq!("0-340-01381-8".parse(), Ok(isbn_10));
//! assert_eq!("978-0-340-01381-6".parse(), Ok(isbn_13));
//! ```
//!
//! [International Standard Book Number]: https://www.isbn-international.org/

use core::fmt;
use core::num::ParseIntError;
use core::str::FromStr;
use arrayvec::ArrayVec;

pub type IsbnResult<T> = Result<T, IsbnError>;

/// An International Standard Book Number, either ISBN10 or ISBN13.
#[derive(Debug, PartialEq, Copy, Clone, Hash)]
pub enum Isbn {
    _10(Isbn10),
    _13(Isbn13),
}

impl fmt::Display for Isbn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Isbn::_10(ref c) => fmt::Display::fmt(c, f),
            Isbn::_13(ref c) => fmt::Display::fmt(c, f),
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
#[derive(Debug, PartialEq, Copy, Clone, Hash)]
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
    /// let isbn10 = Isbn10::new(0, 3, 0, 6, 4, 0, 6, 1, 5, 2);
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
    ) -> IsbnResult<Isbn10> {
        let digits = [a, b, c, d, e, f, g, h, i, j];
        if Isbn10::calculate_check_digit(&digits) == j {
            Ok(Isbn10 { digits })
        } else {
            Err(IsbnError::InvalidChecksum)
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
}

impl fmt::Display for Isbn10 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for x in &self.digits {
            match x {
                10 => write!(f, "X")?,
                _ => write!(f, "{}", x)?,
            }
        }
        Ok(())
    }
}

impl FromStr for Isbn10 {
    type Err = IsbnError;
    fn from_str(s: &str) -> Result<Isbn10, IsbnError> {
        Parser::new(s).read_isbn10()
    }
}

/// 13-digit ISBN format.
#[derive(Debug, PartialEq, Copy, Clone, Hash)]
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
    /// let isbn13 = Isbn13::new(9, 7, 8, 3, 1, 6, 1, 4, 8, 4, 1, 0, 0);
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
    ) -> IsbnResult<Isbn13> {
        let digits = [a, b, c, d, e, f, g, h, i, j, k, l, m];
        if Isbn13::calculate_check_digit(&digits) == m {
            Ok(Isbn13 { digits })
        } else {
            Err(IsbnError::InvalidChecksum)
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
}

impl fmt::Display for Isbn13 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for x in &self.digits {
            write!(f, "{}", x)?;
        }
        Ok(())
    }
}

impl From<Isbn10> for Isbn13 {
    fn from(isbn10: Isbn10) -> Isbn13 {
        let mut v = ArrayVec::<[u8; 13]>::new();
        v.extend([9, 7, 8].iter().cloned());
        v.extend(isbn10.digits[..9].iter().cloned());
        let c = Isbn13::calculate_check_digit(&v);
        let d = isbn10.digits;
        Isbn13::new(
            9, 7, 8, d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], c,
        )
        .unwrap()
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
    /// Failed to validate checksum
    InvalidChecksum,
}

impl From<ParseIntError> for IsbnError {
    fn from(_: ParseIntError) -> Self {
        IsbnError::InvalidDigit
    }
}

#[derive(Debug, Clone)]
struct Parser {
    digits: ArrayVec<[u8; 13]>,
}

impl Parser {
    pub fn new(s: &str) -> Parser {
        let digits = s
            .chars()
            .filter_map(|c| match c {
                '-' => None,
                ' ' => None,
                'X' => Some(10u8),
                _ => Some(c.to_digit(10).unwrap_or(0) as u8),
            })
            .collect();
        Parser { digits }
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
            Isbn13::new(
                d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9], d[10], d[11], d[12],
            )
        } else {
            Err(IsbnError::InvalidDigit)
        }
    }

    fn read_isbn10(&mut self) -> Result<Isbn10, IsbnError> {
        let check_digit = Isbn10::calculate_check_digit(&self.digits);
        if check_digit == *self.digits.last().unwrap() {
            let d = &self.digits;
            Isbn10::new(d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7], d[8], d[9])
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
