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

use serde::de::{self, Unexpected};
use serde::{Deserialize, Deserializer};

use super::XMLModel;

#[derive(Debug, Deserialize)]
pub struct Pics {
    #[serde(rename = "Rows")]
    rows: Rows,
}

#[derive(Debug, Deserialize)]
pub struct Rows {
    #[serde(rename = "Row")]
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Row {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Description")]
    pub description: String,
    #[serde(rename = "Value", deserialize_with = "bool_from_string")]
    pub value: bool,
    #[serde(rename = "Mandatory", deserialize_with = "bool_from_string")]
    pub mandatory: bool,
}

fn bool_from_string<'a, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'a>,
{
    match String::deserialize(deserializer)?.as_ref() {
        "TRUE" => Ok(true),
        "FALSE" => Ok(false),
        s => Err(de::Error::invalid_value(
            Unexpected::Str(s),
            &"TRUE or FALSE",
        )),
    }
}

impl XMLModel<'_> for Pics {
    const PATH: &'static str = "bin/Bluetooth/PICSX";
    const FILE_TYPE: &'static str = "picsx";
}

impl Pics {
    pub fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.rows.iter()
    }
}

#[cfg(test)]
mod test {

    use super::{Pics, Row};
    use serde_xml_rs::from_str;

    #[test]
    fn parse_one() {
        let picsx_xml = r#"
        <PICS>
            <Rows>
                <Row>
                    <Name>TSPC_A2DP_1_1</Name>
                    <Description>Source (C.1)</Description>
                    <Value>FALSE</Value>
                    <Mandatory>FALSE</Mandatory>
                </Row>
            </Rows>
        </PICS>"#;
        let pics: Pics = from_str(&picsx_xml).unwrap();
        let row = Row {
            name: String::from("TSPC_A2DP_1_1"),
            description: String::from("Source (C.1)"),
            value: false,
            mandatory: false,
        };
        assert_eq!(*pics.rows.rows.get(0).unwrap(), row);
    }

    #[test]
    fn parse_two() {
        let picsx_xml = r#"
        <PICS>
            <Rows>
                <Row>
                    <Name>TSPC_A2DP_1_1</Name>
                    <Description>Source (C.1)</Description>
                    <Value>FALSE</Value>
                    <Mandatory>FALSE</Mandatory>
                </Row>
                <Row>
                    <Name>TSPC_A2DP_2_1</Name>
                    <Description>SRC: Initiate Connection Establishment (M)</Description>
                    <Value>TRUE</Value>
                    <Mandatory>TRUE</Mandatory>
              </Row>
            </Rows>
        </PICS>"#;
        let pics: Pics = from_str(&picsx_xml).unwrap();
        let row1 = Row {
            name: String::from("TSPC_A2DP_1_1"),
            description: String::from("Source (C.1)"),
            value: false,
            mandatory: false,
        };
        let row2 = Row {
            name: String::from("TSPC_A2DP_2_1"),
            description: String::from("SRC: Initiate Connection Establishment (M)"),
            value: true,
            mandatory: true,
        };
        assert_eq!(*pics.rows.rows.get(0).unwrap(), row1);
        assert_eq!(*pics.rows.rows.get(1).unwrap(), row2);
    }
}
