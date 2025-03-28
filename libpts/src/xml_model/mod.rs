// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod ets;
mod fn_context;
pub mod picsx;
pub mod pixitx;

use serde::Deserialize;
use serde_xml_rs::Deserializer;

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

    fn parse(profile: &str, wine: &Wine) -> Result<Self, Error> {
        let path = wine.drive_c().join(PTS_PATH).join(Self::PATH).join(format!(
            "{}.{}",
            profile,
            Self::FILE_TYPE
        ));
        let content: String = read_to_string(path.clone()).map_err(|err| {
            Error::FileNotFound(err, String::from(path.to_str().unwrap_or("Unknown")))
        })?;

        // Strip BOM if present as it's not accepted by the xml parser
        let content = content.strip_prefix('\u{feff}').unwrap_or(&*content);

        let mut de =
            Deserializer::new_from_reader(content.as_bytes()).non_contiguous_seq_elements(true);
        let value: Self = Self::deserialize(&mut de).map_err(Error::ParseFailed)?;
        Ok(value)
    }
}
