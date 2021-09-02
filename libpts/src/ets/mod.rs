use serde::Deserialize;
use serde_xml_rs::from_str;

use std::fs::read_to_string;
use std::io;

use evalexpr::error::EvalexprResult;
use evalexpr::eval_boolean_with_context;

use thiserror::Error;

use crate::installer::PTS_PATH;
use crate::wine::Wine;

mod slice_context;
use slice_context::SliceContext;

const ETS_PATH: &'static str = "bin/Bluetooth/Ets/";

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0} {1}")]
    FileNotFound(#[source] io::Error, String),
    #[error("Could not parse xml file: {0}")]
    ParseFailed(#[source] serde_xml_rs::Error),
}

#[derive(Debug, Deserialize)]
pub struct ETS {
    #[serde(rename = "ETSVersion")]
    version: String,
    #[serde(rename = "Profile")]
    profile: Profile,
}

#[derive(Debug, Deserialize)]
struct Profile {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Group", default)]
    groups: Vec<Group>,
}

#[derive(Debug, Deserialize)]
struct Group {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Group", default)]
    groups: Vec<Group>,
    #[serde(rename = "TestCase", default)]
    testcases: Vec<TestCase>,
}

#[derive(Debug, Deserialize)]
struct TestCase {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Mapping")]
    mapping: String,
    #[serde(rename = "Description")]
    description: String,
}

impl TestCase {
    pub fn is_enable(&self, parameters: &[(&str, bool)]) -> EvalexprResult<bool> {
        let mut mapping = self.mapping.replace("AND", "&&");
        mapping = mapping.replace("OR", "||");

        eval_boolean_with_context(&mapping, &SliceContext(parameters))
    }
}

impl Group {
    pub fn get_testcases(&self) -> Box<dyn Iterator<Item = &TestCase> + '_> {
        Box::new(
            self.testcases
                .iter()
                .chain(self.groups.iter().flat_map(|group| group.get_testcases())),
        )
    }
}

impl ETS {
    pub fn parse(profile: String, wine: &Wine) -> Result<Self, Error> {
        let path = wine
            .drive_c()
            .join(PTS_PATH)
            .join(ETS_PATH)
            .join(format!("{}.xml", profile));
        let ets_string: String = read_to_string(path.clone()).map_err(|err| {
            Error::FileNotFound(err, String::from(path.to_str().unwrap_or("Unknown")))
        })?;
        let ets: ETS = from_str(&ets_string).map_err(|err| Error::ParseFailed(err))?;
        Ok(ets)
    }

    pub fn get_valid_testcases<'a>(
        &'a mut self,
        parameters: &'a [(&str, bool)],
    ) -> impl Iterator<Item = String> + 'a {
        self.get_testcases()
            .filter(move |testcase| testcase.is_enable(parameters).unwrap_or(false))
            .map(|testcase| testcase.name.clone())
    }

    fn get_testcases(&mut self) -> impl Iterator<Item = &TestCase> + '_ {
        self.profile
            .groups
            .iter_mut()
            .flat_map(|group| group.get_testcases())
    }
}
