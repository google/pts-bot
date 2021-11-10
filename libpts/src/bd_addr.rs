use serde::{de, Deserialize, Deserializer};
use std::fmt;
use std::ops::Deref;
use std::str::FromStr;

#[derive(Debug, Copy, Clone)]
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
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl FromStr for BdAddr {
    type Err = std::num::ParseIntError;
    fn from_str(src: &str) -> Result<BdAddr, Self::Err> {
        Ok(Self([
            u8::from_str_radix(&src[0..2], 16)?,
            u8::from_str_radix(&src[2..4], 16)?,
            u8::from_str_radix(&src[4..6], 16)?,
            u8::from_str_radix(&src[6..8], 16)?,
            u8::from_str_radix(&src[8..10], 16)?,
            u8::from_str_radix(&src[10..12], 16)?,
        ]))
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
