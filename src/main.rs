use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::net::{Ipv4Addr, Shutdown, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;

use anyhow::{Context, Result};
use dirs;
use libpts::{logger, HCIPort, Interaction, IUT, PTS};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;

mod mmi2grpc;
use mmi2grpc::Mmi2grpc;

const ROOTCANAL_PORT: u16 = 6402;
const GRPC_PORT: u16 = 8999;
const GRPC_SERVER_ROOT_PORT: u16 = 8998;

use termion::{color, style};

struct Host {
    addr: String,
    process: Child,
    mmi2grpc: Mmi2grpc,
}

impl Host {
    fn spawn(command: &Path) -> Result<Self> {
        let process = Command::new(command)
            .arg(format!("--grpc-port={}", GRPC_PORT))
            .arg(format!("--root-server-port={}", GRPC_SERVER_ROOT_PORT)) // Only for RootFacade service in rootservice.proto
            .arg(format!("--rootcanal-port={}", ROOTCANAL_PORT))
            .arg("--blueberry=true")
            .env("ROOTCANAL_PORT", ROOTCANAL_PORT.to_string())
            .spawn()
            .context("Failed to spawn host")?;
        let mmi2grpc = Mmi2grpc::new();
        let addr = mmi2grpc
            .read_local_address()?
            .iter()
            .map(|&c| format!("{:02X}", c))
            .collect::<String>();
        println!("pts-bot: host_addr: {}", addr);
        Ok(Self {
            addr,
            process,
            mmi2grpc,
        })
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        println!("Terminating host");
        self.process.kill().and_then(|_| self.process.wait())
            .expect("Failed to kill host process");
    }
}

impl IUT for Host {
    fn bd_addr(&self) -> &str {
        &self.addr
    }

    fn interact(&mut self, interaction: Interaction) -> String {
        self.mmi2grpc.interact(interaction).unwrap()
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

#[derive(Debug, Deserialize)]
struct Config {
    ics: HashMap<String, bool>,
    ixit: HashMap<String, String>,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "pts-bot",
    about = "Automating PTS tests in virtual environments"
)]
struct Opts {
    /// Config file path
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,

    /// Host binary to use as Implementation Under Test (IUT)
    #[structopt(short, long, parse(from_os_str))]
    host: PathBuf,

    /// All tests under this prefix will be run
    test_prefix: String,
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

    let mut pts =
        PTS::install(cache, connect_to_rootcanal, installer).context("Failed to create PTS")?;

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

    let profile_name = opts
        .test_prefix
        .split_once("/")
        .map(|(profile, _)| profile)
        .unwrap_or(&*opts.test_prefix);

    let profile = pts
        .profile(profile_name)
        .with_context(|| format!("Profile '{}' not found", profile_name))?;

    let tests = profile
        .tests()
        .filter(|test| test.starts_with(&opts.test_prefix))
        .collect::<Vec<_>>();

    println!("Tests: {:?}", tests);

    let result = tests
        .into_iter()
        .map(|test| {
            let mut host = Host::spawn(&opts.host)?;
            let events = profile.run_test(&*test, &mut host);

            let verdict = logger::print(events).context("Runtime Error")?;
            let verdict = verdict.context("No Verdict ?")?;
            println!("Verdict: {}", verdict);
            Ok((test, verdict))
        })
        .collect::<Result<Vec<_>>>()?;

    report_results(result);
    Ok(())
}
