use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::io;
use std::net::{Ipv4Addr, Shutdown, TcpStream};
use std::path::PathBuf;
use std::thread;

use anyhow::{Context, Error, Result};
use dirs;
use libpts::{logger, BdAddr, Interaction, HCI, IUT, PTS};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;

mod mmi2grpc;
use mmi2grpc::Mmi2grpc;

use termion::{color, style};

struct Host {
    addr: BdAddr,
    mmi2grpc: Mmi2grpc,
}

impl Host {
    fn create() -> Result<Self> {
        let mmi2grpc = Mmi2grpc::new();

        println!("Resetting Host ...");
        mmi2grpc.reset()?;

        println!("Reading local address ...");
        let addr = BdAddr::new(
            mmi2grpc
                .read_local_address()?
                .try_into()
                .map_err(|_| Error::msg("Invalid address size"))?,
        );

        println!("local address: {}", addr);
        Ok(Self { addr, mmi2grpc })
    }
}

impl IUT for Host {
    type Err = mmi2grpc::Error;

    fn bd_addr(&self) -> BdAddr {
        self.addr
    }

    fn interact(&mut self, interaction: Interaction) -> std::result::Result<String, Self::Err> {
        self.mmi2grpc.interact(interaction)
    }
}

fn connect_to_rootcanal(port: HCI) {
    let opts = Opts::from_args();
    let rootcanal_port = opts.rootcanal;
    let mut hcitx = port.clone();
    let mut hcirx = port;
    let tcp = TcpStream::connect((Ipv4Addr::LOCALHOST, rootcanal_port)).expect("Connect");
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

    /// All tests under this prefix will be run
    test_prefix: String,

    /// rootcanal hci port
    #[structopt(short, long, default_value = "6402")]
    rootcanal: u16,
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

// FIXME: Use str.split_once when gLinux rustc version >= 1.52.0
pub fn split_once<'a, 'b>(s: &'a str, separator: &'b str) -> Option<(&'a str, &'a str)> {
    let start = s.find(separator)?;
    let end = start + separator.len();
    Some((&s[..start], &s[end..]))
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

    let profile_name = split_once(&opts.test_prefix, "/")
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
            let mut host = Host::create()?;
            let events = profile.run_test(&*test, &mut host, connect_to_rootcanal, None);

            let verdict = logger::print(events).context("Runtime Error")?;
            let verdict = verdict.context("No Verdict ?")?;
            println!("Verdict: {}", verdict);
            Ok((test, verdict))
        })
        .collect::<Result<Vec<_>>>()?;

    report_results(result);
    Ok(())
}
