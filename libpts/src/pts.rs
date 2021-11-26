use std::io::{self, BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Stdio};

use serde::Deserialize;
use serde_json;
use serde_repr::Deserialize_repr;

use thiserror::Error;

use crate::bd_addr::BdAddr;
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

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Addr {
        value: BdAddr,
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
    Raw(String),
}

#[derive(Debug, Error)]
pub enum Error<E> {
    #[error("IO {0}")]
    IO(#[source] io::Error),
    #[error(transparent)]
    ImplicitSend(#[from] E),
}

pub struct Messages<'wine, F> {
    process: Child,
    // The port need to live as much time as the process
    _port: WineHCIPort<'wine>,
    implicit_send: F,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl<'wine, F, E> Iterator for Messages<'wine, F>
where
    F: FnMut(&str, MMIStyle) -> Result<String, E>,
{
    type Item = Result<Message, Error<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();

        match self.stdout.read_line(&mut line) {
            Ok(0) => None,
            Ok(_size) => {
                if let Ok(message) = serde_json::from_str(&line) {
                    if let Message::ImplicitSend {
                        ref description,
                        ref style,
                    } = message
                    {
                        let result = match (self.implicit_send)(description, style.clone()) {
                            Ok(answer) => {
                                write!(&mut self.stdin, "{}\n", answer).map_err(Error::IO)
                            }
                            Err(e) => Err(Error::ImplicitSend(e)),
                        };

                        if let Err(e) = result {
                            return Some(Err(e));
                        }
                    }

                    Some(Ok(message))
                } else {
                    Some(Ok(Message::Raw(line)))
                }
            }
            Err(e) => Some(Err(Error::IO(e))),
        }
    }
}

impl<'wine, F> std::ops::Drop for Messages<'wine, F> {
    fn drop(&mut self) {
        // TODO: handle failure
        let _ = self.process.kill().and_then(|_| self.process.wait());
    }
}

pub fn run<'wine, 'a, F, E>(
    port: WineHCIPort<'wine>,
    profile: &str,
    test_case: &str,
    parameters: impl Iterator<Item = (&'a str, &'a str, &'a str)>,
    audio_output_path: Option<&str>,
    implicit_send: F,
) -> Messages<'wine, F>
where
    F: FnMut(&str, MMIStyle) -> Result<String, E>,
{
    let wine = &port.wine;
    let dir = wine.drive_c().join(PTS_PATH).join("bin");

    let mut process = wine
        .command("server.exe", false, audio_output_path)
        .current_dir(dir)
        .arg(port.com.as_ref().unwrap().to_uppercase())
        .arg(profile)
        .arg(test_case)
        // FIXME: remove the to_vec() when gLinux rustc version >= 1.53.0
        .args(parameters.flat_map(|(key, value_type, value)| [key, value_type, value].to_vec()))
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
