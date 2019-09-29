//! Lightweight domestic number
//!
//! Some of this was inspired by:
//! https://github.com/1aim/rust-phonenumber
//! Will switch to it if no_std support lands.

use core::convert::TryFrom;
use core::fmt;
use nom::character::complete::digit1;
use nom::error::ErrorKind;
use nom::{do_parse, named, tag, take};

// TODO - validator, error mapping

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    NoNumber,
    /*TooShort,
     *TooLong, */
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PhoneNumber {
    area_code: u16,
    exchange: u16,
    line_number: u16,
}

impl PhoneNumber {
    pub fn new(area_code: u16, exchange: u16, line_number: u16) -> Self {
        PhoneNumber {
            area_code,
            exchange,
            line_number,
        }
    }

    pub fn from_utf8<'a>(s: &'a str) -> Result<Self, Error> {
        // TODO - error mapping
        let (_remaining, num) = parse_num(s)
            .or(parse_ud_num(s).map_err(|_| Error::NoNumber))
            .map_err(|_| Error::NoNumber)?;

        Ok(num)
    }

    pub fn area_code(&self) -> u16 {
        self.area_code
    }

    pub fn exchange(&self) -> u16 {
        self.exchange
    }

    pub fn line_number(&self) -> u16 {
        self.line_number
    }

    pub fn is_valid(&self) -> bool {
        // TODO
        true
    }
}

named!(
    parse_num<&str, PhoneNumber>,
    do_parse!(
        first: digit1
            >> tag!("-")
            >> second: digit1
            >> tag!("-")
            >> third: digit1
            >> (PhoneNumber {
                area_code: u16::from_str_radix(first, 10)
                    .map_err(|_| nom::Err::Error((first, ErrorKind::Digit)))?,
                exchange: u16::from_str_radix(second, 10)
                    .map_err(|_| nom::Err::Error((first, ErrorKind::Digit)))?,
                line_number: u16::from_str_radix(third, 10)
                    .map_err(|_| nom::Err::Error((first, ErrorKind::Digit)))?,
            })
    )
);

// TODO - combine these parsers
named!(
    parse_ud_num<&str, PhoneNumber>,
    do_parse!(
        first: take!(3)
            >> second: take!(3)
            >> third: take!(4)
            >> (PhoneNumber {
                area_code: u16::from_str_radix(first, 10)
                    .map_err(|_| nom::Err::Error((first, ErrorKind::Digit)))?,
                exchange: u16::from_str_radix(second, 10)
                    .map_err(|_| nom::Err::Error((first, ErrorKind::Digit)))?,
                line_number: u16::from_str_radix(third, 10)
                    .map_err(|_| nom::Err::Error((first, ErrorKind::Digit)))?,
            })
    )
);

impl<'a> TryFrom<&'a str> for PhoneNumber {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<Self, Self::Error> {
        PhoneNumber::from_utf8(s)
    }
}

impl fmt::Display for PhoneNumber {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:#03}-{:#03}-{:#04}",
            self.area_code(),
            self.exchange(),
            self.line_number(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::{debug, trace};

    #[test_case]
    fn new_number() {
        trace!("new_number");
        let num = PhoneNumber::new(222, 333, 1234);
        debug!("Created {}", num);
        assert_eq!(num.is_valid(), true);
        assert_eq!(num.area_code(), 222);
        assert_eq!(num.exchange(), 333);
        assert_eq!(num.line_number(), 1234);
    }

    #[test_case]
    fn delimited_parse() {
        trace!("delimited_parse");
        let num = PhoneNumber::new(333, 222, 1234);
        let res = PhoneNumber::try_from("333-222-1234");
        assert_eq!(res, Ok(num));
    }

    #[test_case]
    fn undelimited_parse() {
        trace!("undelimited_parse");
        let num = PhoneNumber::new(123, 456, 1234);
        let res = PhoneNumber::try_from("1234561234");
        assert_eq!(res, Ok(num));
    }
}
