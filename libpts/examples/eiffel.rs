use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{BufRead, Write};
use std::net::{Ipv4Addr, Shutdown, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, ChildStdin, Command, Stdio};
use std::thread;

use anyhow::{Context, Result};
use dirs;
use libpts::{logger, BdAddr, Interaction, MMIStyle, HCI, IUT, PTS};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;
use termion::{color, style};

const ROOTCANAL_PORT: u16 = 6402;

struct Eiffel {
    addr: BdAddr,
    process: Child,
    lines: io::Lines<io::BufReader<ChildStderr>>,
    stdin: ChildStdin,
}

impl Eiffel {
    fn spawn(command: &Path, test: &String) -> Result<Self> {
        // Save the record trace in a file named after the test being run.
        let eiffel_record_file = env::var("EIFFEL_RECORD_DIR").map_or("".to_owned(), |d| {
            format!("{}/{}.pcap", d, test.replace("/", "_"))
        });

        let mut process = Command::new(command)
            .arg("any")
            .env("EIFFEL_RECORD_FILE", eiffel_record_file)
            .env("ROOTCANAL_PORT", ROOTCANAL_PORT.to_string())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()
            .context("Failed to spawn eiffel")?;

        let mut lines = io::BufReader::new(process.stderr.take().unwrap()).lines();

        let addr = lines.next().unwrap().unwrap().parse().unwrap();

        let stdin = process.stdin.take().unwrap();

        Ok(Self {
            process,
            addr,
            lines,
            stdin,
        })
    }
}

impl Drop for Eiffel {
    fn drop(&mut self) {
        println!("Terminating eiffel");
        let _ = self.process.kill().and_then(|_| self.process.wait());
    }
}

impl IUT for Eiffel {
    fn bd_addr(&self) -> BdAddr {
        self.addr
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

fn connect_to_rootcanal(port: HCI) {
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

#[derive(Debug, Deserialize)]
struct Config {
    ics: HashMap<String, bool>,
    ixit: HashMap<String, String>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "libpts", about = "libpts eiffel runner")]
struct Opts {
    /// Profile to test, eg "A2DP", "L2CAP", ...
    #[structopt(short, long)]
    profile: String,

    /// Config file path
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,

    /// Eiffel pts binary to use as Implementation Under Test (IUT)
    #[structopt(short, long, parse(from_os_str))]
    eiffel: PathBuf,
}

fn report_results(results: Vec<(String, String)>) {
    println!();
    for execution in results.iter() {
        let result = match &*execution.1 {
            "PASS" => format!("{}✔{}", color::Fg(color::Green), style::Reset),
            "FAIL" => format!("{}✘{}", color::Fg(color::Red), style::Reset),
            "INCONC" => format!("{}?{}", color::Fg(color::Yellow), style::Reset),
            _ => format!("{}?{}", color::Fg(color::LightWhite), style::Reset),
        };
        println!(
            "\t{}{}{}{}: {}",
            style::Bold,
            color::Fg(color::LightWhite),
            execution.0,
            style::Reset,
            result
        );
    }
    let total = results.len();
    let success = results.iter().filter(|e| e.1 == "PASS").count();
    let failed = results.iter().filter(|e| e.1 == "FAIL").count();
    let inconc = results.iter().filter(|e| e.1 == "INCONC").count();
    println!(
        "\n{}Total{}: {}, {} Success, {} Failed, {} Inconc",
        style::Bold,
        style::Reset,
        total,
        success,
        failed,
        inconc
    );
}

fn main() -> Result<()> {
    let opts = Opts::from_args();

    let mut config = dirs::config_dir().context("Failed to get config dir")?;
    config.push("pts");

    let installer = File::open(config.join("pts_setup_8_0_3.exe")).with_context(|| {
        format!(
            "Installer (pts_setup_8_0_3.exe) not found in {}, {}",
            config.display(),
            "download it from the SIG website and add it",
        )
    })?;

    let mut cache = dirs::cache_dir().context("Failed to get cache dir")?;
    cache.push("pts");

    let mut pts = PTS::install(cache, installer).context("Failed to create PTS")?;

    if let Some(ref config_path) = opts.config {
        let config_file = File::open(config_path).context("Failed to open config file")?;
        let config: Config = serde_json::from_reader(io::BufReader::new(config_file))
            .context("Failed to parse config")?;

        for (ics, value) in config.ics {
            pts.set_ics(&*ics, value);
        }

        for (ixit, value) in config.ixit {
            pts.set_ixit(&*ixit, &*value);
        }
    }

    let profile = pts
        .profile(&*opts.profile)
        .with_context(|| format!("Profile '{}' not found", &opts.profile))?;

    println!("Tests: {:?}", profile.tests().collect::<Vec<_>>());

    let results = profile
        .tests()
        .map(|test| {
            let mut eiffel = Eiffel::spawn(&opts.eiffel, &test)?;
            let events = profile.run_test(&*test, &mut eiffel, connect_to_rootcanal);

            let verdict = logger::print(events).context("Runtime Error")?;
            let verdict = verdict.context("No Verdict ?")?;

            println!("Verdict: {}", verdict);
            Ok((test, verdict))
        })
        .collect::<Result<Vec<_>>>()?;

    report_results(results);

    Ok(())
}
