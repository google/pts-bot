use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Stdio;
use std::process::{Child, ChildStdin, ChildStdout};

use serde::Deserialize;
use serde_json;

use crate::hci::WineHCIPort;
use crate::installer::PTS_PATH;
use crate::wine::Wine;

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Message {
    Addr {
        value: String,
    },
    Dongle {
        message: String,
    },
    ImplicitSend {
        description: String,
    },
    Log {
        time: String,
        description: String,
        message: String,
    },
}

pub struct Messages<'a, F>
where
    F: Fn(&str) -> String,
{
    process: Child,
    port: WineHCIPort<'a>,
    implicit_send: F,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl<'a, F> Iterator for Messages<'a, F>
where
    F: Fn(&str) -> String,
{
    type Item = std::io::Result<Message>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();

        match self.stdout.read_line(&mut line) {
            Ok(0) => None,
            Ok(_size) => {
                let message: Message = serde_json::from_str(&line).unwrap();

                if let Message::ImplicitSend { ref description } = message {
                    let answer = (self.implicit_send)(description);
                    write!(&mut self.stdin, "{}\n", answer);
                }

                Some(Ok(message))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

impl<'a, F> std::ops::Drop for Messages<'a, F>
where
    F: Fn(&str) -> String,
{
    fn drop(&mut self) {
        // TODO: handle failure
        let _ = self.process.kill().and_then(|_| self.process.wait());
    }
}

pub fn run<'a, 'b, F>(
    wine: &Wine,
    profile: &str,
    test_case: &str,
    parameters: impl Iterator<Item = &'b (&'b str, &'b str, &'b str)>,
    implicit_send: F,
    port: WineHCIPort<'a>,
) -> Messages<'a, F>
where
    F: Fn(&str) -> String,
{
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
        port,
        implicit_send,
        stdin,
        stdout,
    }
}
