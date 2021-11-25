use serde::Deserialize;

use evalexpr::error::EvalexprResult;
use evalexpr::eval_boolean_with_context;

use super::fn_context::FnContext;

use super::XMLModel;

#[derive(Debug, Deserialize)]
pub struct ETS {
    #[serde(rename = "ETSVersion")]
    version: Option<String>,
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
    #[serde(rename = "Mapping", default)]
    mapping: String,
    #[serde(rename = "Description", default)]
    description: String,
}

impl TestCase {
    pub fn is_enabled<F: Fn(&str) -> Option<bool>>(&self, get_value: &F) -> EvalexprResult<bool> {
        let mut mapping = self.mapping.replace("AND", "&&");
        mapping = mapping.replace("OR", "||");

        eval_boolean_with_context(&mapping, &FnContext(get_value))
    }
}

impl Group {
    pub fn testcases(&self) -> Box<dyn Iterator<Item = &TestCase> + '_> {
        Box::new(
            self.testcases
                .iter()
                .chain(self.groups.iter().flat_map(|group| group.testcases())),
        )
    }
}

impl<'a> XMLModel<'a> for ETS {
    const PATH: &'static str = "bin/Bluetooth/Ets/";
}

impl ETS {
    pub fn enabled_testcases<'a, F: 'a + Fn(&str) -> Option<bool>>(
        &'a self,
        get_value: F,
    ) -> impl Iterator<Item = String> + 'a {
        self.testcases()
            .filter(move |testcase| testcase.is_enabled(&get_value).unwrap_or(false))
            .map(|testcase| testcase.name.clone())
    }

    fn testcases(&self) -> impl Iterator<Item = &TestCase> + '_ {
        self.profile
            .groups
            .iter()
            .flat_map(|group| group.testcases())
    }
}
