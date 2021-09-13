use std::env;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::net::{Ipv4Addr, Shutdown, TcpStream};
use std::process::{Child, ChildStderr, ChildStdin, Command, Stdio};
use std::thread;

use dirs;
use libpts::{Event, HCIPort, Interaction, MMIStyle, Message, IUT, PTS};

const ROOTCANAL_PORT: u16 = 6402;

struct Eiffel {
    addr: String,
    process: Child,
    lines: io::Lines<io::BufReader<ChildStderr>>,
    stdin: ChildStdin,
}

impl Eiffel {
    fn spawn(command: &str) -> Self {
        let mut process = Command::new(command)
            .arg("any")
            .env("ROOTCANAL_PORT", ROOTCANAL_PORT.to_string())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .expect("Eiffel Spawn failed");

        let mut lines = io::BufReader::new(process.stderr.take().unwrap()).lines();

        let addr = lines
            .next()
            .unwrap()
            .unwrap()
            .replace(":", "")
            .to_uppercase();
        let stdin = process.stdin.take().unwrap();

        Self {
            process,
            addr,
            lines,
            stdin,
        }
    }
}

impl Drop for Eiffel {
    fn drop(&mut self) {
        let _ = self.process.kill().and_then(|_| self.process.wait());
    }
}

impl IUT for Eiffel {
    fn bd_addr(&self) -> &str {
        &self.addr
    }

    fn interact(&mut self, interaction: Interaction) -> String {
        let values = match interaction.style {
            MMIStyle::OkCancel1 | MMIStyle::OkCancel2 => "2|OK|Cancel",
            MMIStyle::Ok => "1|OK",
            MMIStyle::YesNo1 => "2|Yes|No",
            MMIStyle::YesNoCancel1 => "3|Yes|No|Cancel",
            MMIStyle::AbortRetry1 => "3|Abort|Retry|Ignore",
            MMIStyle::Edit1 => "0",
            MMIStyle::Edit2 => unreachable!(),
        };

        write!(
            &mut self.stdin,
            "any|{addr}|{id}|{test}|{values}|{description}\0",
            addr = interaction.pts_addr,
            id = interaction.id,
            test = interaction.test,
            values = values,
            description = interaction.description
        )
        .unwrap();

        self.stdin.flush().unwrap();

        let answer = self.lines.next().unwrap().unwrap();

        answer
    }
}

fn connect_to_rootcanal(port: HCIPort) {
    let mut hcitx = port.clone();
    let mut hcirx = port;
    let tcp = TcpStream::connect((Ipv4Addr::LOCALHOST, ROOTCANAL_PORT)).expect("Connect");
    let mut tcptx = tcp.try_clone().expect("Clone");
    let mut tcprx = tcp;
    thread::spawn(move || {
        io::copy(&mut hcitx, &mut tcprx).expect("HCI TX");
        println!("HCI TX ended");
        tcprx.shutdown(Shutdown::Both).expect("HCI shutdown");
    });
    thread::spawn(move || {
        io::copy(&mut tcptx, &mut hcirx).expect("HCI RX");
        println!("HCI RX ended");
    });
}

fn main() {
    let mut cache = dirs::cache_dir().expect("No cache dir");
    cache.push("pts");

    let pts = PTS::install(cache, connect_to_rootcanal).expect("PTS");

    let mut eiffel = Eiffel::spawn(&*env::args().nth(1).unwrap());

    let profile = pts.profile("A2DP").unwrap();

    println!("Tests {:?}", profile.tests().collect::<Vec<_>>());

    for test in profile.tests() {
        let events = profile.run_test(&*test, &mut eiffel);

        for event in events {
            match event.unwrap() {
                Event::EnterTestStep(test_step, num) => {
                    println!("{:<1$}{test_step}", "", num * 2, test_step = test_step)
                }
                Event::Message(
                    Message::Log {
                        ref message,
                        ref description,
                        ..
                    },
                    num,
                ) => {
                    println!(
                        "{:<1$}- {description}{message}",
                        "",
                        num * 2,
                        description = description.trim(),
                        message = message.trim()
                    );
                }
                _ => {}
            }
        }

        break;
    }
}
