use std::convert::TryFrom;
use std::fmt::Debug;

use anyhow::anyhow;

use termion::{color, style};

pub enum TestResult {
    Pass,
    Fail,
    Inconclusive,
    None,
    Error(anyhow::Error),
}

impl<S: PartialEq<str> + Debug> TryFrom<Result<Option<S>, anyhow::Error>> for TestResult {
    type Error = anyhow::Error;

    fn try_from(value: Result<Option<S>, anyhow::Error>) -> Result<Self, anyhow::Error> {
        match value {
            Ok(Some(ref s)) if s == "PASS" => Ok(TestResult::Pass),
            Ok(Some(ref s)) if s == "FAIL" => Ok(TestResult::Fail),
            Ok(Some(ref s)) if s == "INCONC" => Ok(TestResult::Inconclusive),
            Ok(Some(ref s)) if s == "NONE" => Ok(TestResult::None),
            Ok(None) => Ok(TestResult::None),
            Err(e) => Ok(TestResult::Error(e)),
            value => Err(anyhow!("unknown test result {:?}", value)),
        }
    }
}

pub struct TestExecution {
    pub name: String,
    pub result: TestResult,
}

pub fn report(results: Vec<TestExecution>) {
    println!();
    for execution in results.iter() {
        print!("  ");

        match execution.result {
            TestResult::Pass => print!(" {}✔{} ", color::Fg(color::Green), style::Reset),
            TestResult::Fail => print!(" {}✘{} ", color::Fg(color::Red), style::Reset),
            TestResult::Inconclusive => print!(" {}?{} ", color::Fg(color::Yellow), style::Reset),
            TestResult::None => print!(
                "{}{}N/A{}",
                style::Bold,
                color::Fg(color::Cyan),
                style::Reset
            ),
            TestResult::Error(_) => print!(" ☠️  "),
        };

        println!(
            "  {}{}{}{}",
            style::Bold,
            color::Fg(color::LightWhite),
            execution.name,
            style::Reset,
        );

        if let TestResult::Error(ref e) = execution.result {
            println!("{:?}", e);
        }
    }
    let total = results.len();
    let success = results
        .iter()
        .filter(|e| matches!(e.result, TestResult::Pass))
        .count();
    let failed = results
        .iter()
        .filter(|e| matches!(e.result, TestResult::Fail))
        .count();
    let inconc = results
        .iter()
        .filter(|e| matches!(e.result, TestResult::Inconclusive))
        .count();

    println!(
        "\n{}Total{}: {}, {} Success, {} Failed, {} Inconclusive",
        style::Bold,
        style::Reset,
        total,
        success,
        failed,
        inconc
    );
}
