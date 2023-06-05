use serde::Deserialize;

use super::XMLModel;

#[derive(Debug, Deserialize)]
pub struct Pixit {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Version")]
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

impl<'a> XMLModel<'a> for Pixit {
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
