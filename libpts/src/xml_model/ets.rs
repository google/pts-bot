use serde::Deserialize;

use evalexpr::error::EvalexprResult;
use evalexpr::eval_boolean_with_context;

use super::slice_context::SliceContext;

use super::XMLModel;

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

impl<'a> XMLModel<'a> for ETS {
    const PATH: &'static str = "bin/Bluetooth/Ets/";
}

impl ETS {
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
