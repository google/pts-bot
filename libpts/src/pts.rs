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

use std::io::Write;
use std::process::{Child, Stdio};

use async_io::Async;

use futures_lite::{io::BufReader, AsyncBufReadExt, Stream, StreamExt};

use serde::Deserialize;
use serde_repr::Deserialize_repr;

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

#[derive(Deserialize_repr, PartialEq, Debug, Clone, Copy)]
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
        #[allow(dead_code)]
        description: String,
        message: String,
        logtype: LogType,
    },
    Raw(String),
}

pub struct Server<'wine>(Child, #[allow(dead_code)] WineHCIPort<'wine>);

impl<'wine> Server<'wine> {
    pub fn spawn<'a>(
        port: WineHCIPort<'wine>,
        profile: &str,
        test_case: &str,
        parameters: impl Iterator<Item = (&'a str, &'a str, &'a str)>,
        audio_output_path: Option<&str>,
    ) -> Self {
        let wine = &port.wine;
        let dir = wine.drive_c().join(PTS_PATH).join("bin");

        let process = wine
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
        Self(process, port)
    }

    pub fn into_parts(
        mut self,
    ) -> (
        impl Stream<Item = std::io::Result<Message>> + 'wine,
        impl FnMut(&str) + 'wine,
    ) {
        let stdout = self.0.stdout.take().unwrap();
        let stdout = BufReader::new(Async::new(stdout).unwrap());

        (
            stdout.lines().map(|result| {
                result.map(|line| {
                    if let Ok(message) = serde_json::from_str(&line) {
                        message
                    } else {
                        Message::Raw(line)
                    }
                })
            }),
            move |answer| {
                // TODO(b/239749174): Handle result
                let _ = writeln!(self.0.stdin.as_mut().unwrap(), "{}", answer);
            },
        )
    }
}

impl std::ops::Drop for Server<'_> {
    fn drop(&mut self) {
        // TODO: handle failure
        let _ = self.0.kill().and_then(|_| self.0.wait());
    }
}
