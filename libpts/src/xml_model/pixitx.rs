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

use serde::Deserialize;

use super::XMLModel;

#[derive(Debug, Deserialize)]
pub struct Pixit {
    #[serde(rename = "Name")]
    #[allow(dead_code)]
    pub name: String,
    #[serde(rename = "Version")]
    #[allow(dead_code)]
    pub version: String,
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
    // GAP.pixitx contains entries with duplicates
    // of the <Type> entrie.
    #[serde(rename = "Type")]
    pub value_type: Vec<String>,
    #[serde(rename = "Value")]
    pub value: String,
}

impl XMLModel<'_> for Pixit {
    const PATH: &'static str = "bin/Bluetooth/PIXITX";
    const FILE_TYPE: &'static str = "pixitx";
}

impl Pixit {
    pub fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.rows.iter()
    }
}

#[cfg(test)]
mod test {

    use super::{Pixit, Row};
    use serde_xml_rs::from_str;

    #[test]
    fn parse_one() {
        let pixitx_xml = r#"
        <PIXIT>
            <Name>A2DP</Name>
            <Version></Version>
            <Rows>
                <Row>
                    <Name>TSPX_security_enabled</Name>
                    <Description>Whether security is required for establishing connections. (Default: FALSE)</Description>
                    <Type>BOOLEAN</Type>
                    <Value>FALSE</Value>
                </Row>
            </Rows>
        </PIXIT>"#;
        let pixitx: Pixit = from_str(&pixitx_xml).unwrap_or_else(|err| {
            println!("error: {}", err);
            panic!();
        });
        let row: Row = Row {
            name: String::from("TSPX_security_enabled"),
            description: String::from(
                "Whether security is required for establishing connections. (Default: FALSE)",
            ),
            value_type: vec![String::from("BOOLEAN")],
            value: String::from("FALSE"),
        };
        assert_eq!(pixitx.rows.rows.get(0), Some(&row));
    }

    #[test]
    fn parse_two() {
        let pixitx_xml = r#"
        <PIXIT>
            <Name>A2DP</Name>
            <Version></Version>
            <Rows>
                <Row>
                    <Name>TSPX_security_enabled</Name>
                    <Description>Whether security is required for establishing connections. (Default: FALSE)</Description>
                    <Type>BOOLEAN</Type>
                    <Value>FALSE</Value>
                </Row>
                <Row>
                    <Name>TSPX_bd_addr_iut</Name>
                    <Description>The unique 48-bit Bluetooth device address (BD_ADDR) of the IUT. This was filled in during workspace creation.</Description>
                    <Type>OCTETSTRING</Type>
                    <Value>000272406FAC</Value>
                </Row>
            </Rows>
        </PIXIT>"#;
        let pixitx: Pixit = from_str(&pixitx_xml).unwrap_or_else(|err| {
            println!("error: {}", err);
            panic!();
        });
        let row: Row = Row {
            name: String::from("TSPX_security_enabled"),
            description: String::from(
                "Whether security is required for establishing connections. (Default: FALSE)",
            ),
            value_type: vec![String::from("BOOLEAN")],
            value: String::from("FALSE"),
        };
        let row1: Row = Row {
            name: String::from("TSPX_bd_addr_iut"),
            description: String::from("The unique 48-bit Bluetooth device address (BD_ADDR) of the IUT. This was filled in during workspace creation."),
            value_type: vec![String::from("OCTETSTRING")],
            value: String::from("000272406FAC"),
        };
        assert_eq!(pixitx.rows.rows.get(0), Some(&row));
        assert_eq!(pixitx.rows.rows.get(1), Some(&row1));
    }
}
