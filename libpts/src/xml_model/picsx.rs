use serde::de::{self, Unexpected};
use serde::{Deserialize, Deserializer};

use super::XMLModel;

#[derive(Debug, Deserialize)]
pub struct PICS {
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
    name: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Value", deserialize_with = "bool_from_string")]
    value: bool,
    #[serde(rename = "Mandatory", deserialize_with = "bool_from_string")]
    mandatory: bool,
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

impl<'a> XMLModel<'a> for PICS {
    const PATH: &'static str = "bin/Bluetooth/PICSX";
    const FILE_TYPE: &'static str = "picsx";
}

impl Rows {
    pub fn get_parameters(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }
}

#[cfg(test)]
mod test {

    use super::{Row, PICS};
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
        let pics: PICS = from_str(&picsx_xml).unwrap();
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
        let pics: PICS = from_str(&picsx_xml).unwrap();
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
