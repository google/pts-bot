use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Stdio;
use std::process::{Child, ChildStdin, ChildStdout};

use serde::Deserialize;
use serde_json;
use serde_repr::Deserialize_repr;

use crate::hci::WineHCIPort;
use crate::installer::PTS_PATH;

#[derive(Deserialize_repr, PartialEq, Debug)]
#[repr(u8)]
pub enum LogType {
    GeneralText = 0,
    StartTestCase = 1,
    TestCaseEnded = 2,
    StartDefault = 3,
    DefaultEnded = 4,
    FinalVerdict = 5,
    PreliminaryVerdict = 6,
    Timeout = 7,
    Assignment = 8,
    StartTimer = 9,
    StopTimer = 10,
    CancelTimer = 11,
    ReadTimer = 12,
    Attach = 13,
    ImplicitSend = 14,
    Goto = 15,
    TimedOutTimer = 16,
    Error = 17,
    Create = 18,
    Done = 19,
    Activate = 20,
    Message = 21,
    LineMatched = 22,
    LineNotMatched = 23,
    SendEvent = 24,
    ReceiveEvent = 25,
    OtherwiseEvent = 26,
    ReceivedOnPco = 27,
    MatchFailed = 28,
    CoordinationMessage = 29,
}

#[derive(Deserialize_repr, PartialEq, Debug, Clone)]
#[repr(u32)]
pub enum MMIStyle {
    OkCancel1 = 0x11041,
    OkCancel2 = 0x11141,
    Ok = 0x11040,
    YesNo1 = 0x11044,
    YesNoCancel1 = 0x11043,
    AbortRetry1 = 0x11042,
    Edit1 = 0x12040,
    Edit2 = 0x12140,
}

pub type BdAddr = String;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Addr {
        value: BdAddr,
    },
    Dongle {
        message: String,
    },
    ImplicitSend {
        description: String,
        style: MMIStyle,
    },
    Log {
        time: String,
        description: String,
        message: String,
        logtype: LogType,
    },
}

pub struct Messages<'wine, F>
where
    F: FnMut(&str, MMIStyle) -> String,
{
    process: Child,
    // The port need to live as much time as the process
    _port: WineHCIPort<'wine>,
    implicit_send: F,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl<'wine, F> Iterator for Messages<'wine, F>
where
    F: FnMut(&str, MMIStyle) -> String,
{
    type Item = std::io::Result<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();

        match self.stdout.read_line(&mut line) {
            Ok(0) => None,
            Ok(_size) => {
                let message: Message = serde_json::from_str(&line).unwrap();

                if let Message::ImplicitSend {
                    ref description,
                    ref style,
                } = message
                {
                    let answer = (self.implicit_send)(description, style.clone());
                    write!(&mut self.stdin, "{}\n", answer);
                }

                Some(Ok(message))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'wine, F> std::ops::Drop for Messages<'wine, F>
where
    F: FnMut(&str, MMIStyle) -> String,
{
    fn drop(&mut self) {
        // TODO: handle failure
        let _ = self.process.kill().and_then(|_| self.process.wait());
    }
}

pub fn run<'wine, 'a, F>(
    port: WineHCIPort<'wine>,
    profile: &str,
    test_case: &str,
    parameters: impl Iterator<Item = &'a (&'a str, &'a str, &'a str)>,
    implicit_send: F,
) -> Messages<'wine, F>
where
    F: FnMut(&str, MMIStyle) -> String,
{
    let wine = &port.wine;
    let dir = wine.drive_c().join(PTS_PATH).join("bin");

    let mut process = wine
        .command("server.exe", false)
        .current_dir(dir)
        .arg(port.com.as_ref().unwrap().to_uppercase())
        .arg(profile)
        .arg(test_case)
        .args(parameters.flat_map(|(key, value_type, value)| [key, value_type, value]))
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to launch server");

    let stdout = process.stdout.take().unwrap();
    let stdin = process.stdin.take().unwrap();
    let stdout = BufReader::new(stdout);

    Messages {
        process,
        _port: port,
        implicit_send,
        stdin,
        stdout,
    }
}
