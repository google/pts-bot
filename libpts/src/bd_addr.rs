use serde::{de, Deserialize, Deserializer};
use thiserror::Error;

use std::fmt;
use std::num::ParseIntError;
use std::ops::Deref;
use std::str::FromStr;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct BdAddr([u8; 6]);

impl BdAddr {
    pub const NULL: BdAddr = BdAddr([0; 6]);
}

impl BdAddr {
    pub fn new(value: [u8; 6]) -> Self {
        Self(value)
    }
}

impl fmt::Display for BdAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if f.alternate() {
            write!(
                f,
                "{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
                self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
            )
        } else {
            write!(
                f,
                "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
            )
        }
    }
}

#[derive(PartialEq, Debug, Error)]
pub enum ParseBdAddrError {
    #[error("Invalid Byte in Address ({0})")]
    InvalidByte(#[from] ParseIntError),
    #[error("Unknown Address Format")]
    UnknownFormat,
}

impl FromStr for BdAddr {
    type Err = ParseBdAddrError;
    fn from_str(src: &str) -> Result<BdAddr, Self::Err> {
        match src.len() {
            12 => Ok(Self([
                u8::from_str_radix(&src[0..2], 16)?,
                u8::from_str_radix(&src[2..4], 16)?,
                u8::from_str_radix(&src[4..6], 16)?,
                u8::from_str_radix(&src[6..8], 16)?,
                u8::from_str_radix(&src[8..10], 16)?,
                u8::from_str_radix(&src[10..12], 16)?,
            ])),
            17 if src.chars().skip(2).step_by(3).all(|c| c == ':') => Ok(Self([
                u8::from_str_radix(&src[0..2], 16)?,
                u8::from_str_radix(&src[3..5], 16)?,
                u8::from_str_radix(&src[6..8], 16)?,
                u8::from_str_radix(&src[9..11], 16)?,
                u8::from_str_radix(&src[12..14], 16)?,
                u8::from_str_radix(&src[15..17], 16)?,
            ])),
            _ => Err(ParseBdAddrError::UnknownFormat),
        }
    }
}

impl Deref for BdAddr {
    type Target = [u8; 6];

    fn deref(&self) -> &[u8; 6] {
        &self.0
    }
}

impl<'de> Deserialize<'de> for BdAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod test {
    use super::BdAddr;

    #[test]
    fn test_from_str_with_colons() {
        assert_eq!(
            "11:22:33:44:55:66".parse(),
            Ok(BdAddr::new([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]))
        );
    }

    #[test]
    fn test_from_str_without_colons() {
        assert_eq!(
            "112233445566".parse(),
            Ok(BdAddr::new([0x11, 0x22, 0x33, 0x44, 0x55, 0x66]))
        );
    }
}
