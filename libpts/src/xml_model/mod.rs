pub mod ets;
pub mod picsx;
pub mod pixitx;
mod slice_context;

use serde::Deserialize;
use serde_xml_rs::from_str;

use crate::installer::PTS_PATH;
use crate::wine::Wine;

use thiserror::Error;

use std::fs::read_to_string;
use std::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0} {1}")]
    FileNotFound(#[source] io::Error, String),
    #[error("Could not parse xml file: {0}")]
    ParseFailed(#[source] serde_xml_rs::Error),
}

pub trait XMLModel<'a>: Deserialize<'a> {
    const PATH: &'static str;
    const FILE_TYPE: &'static str = "xml";

    fn parse(profile: String, wine: &Wine) -> Result<Self, Error> {
        let path = wine.drive_c().join(PTS_PATH).join(Self::PATH).join(format!(
            "{}.{}",
            profile,
            Self::FILE_TYPE
        ));
        let content: String = read_to_string(path.clone()).map_err(|err| {
            Error::FileNotFound(err, String::from(path.to_str().unwrap_or("Unknown")))
        })?;
        let value: Self = from_str(&content).map_err(|err| Error::ParseFailed(err))?;
        Ok(value)
    }
}
