use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs::File;
use std::io::BufReader;
use std::net::{Ipv4Addr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Error, Result};
use dirs;
use libpts::{logger, BdAddr, Interaction, RunError, HCI, PTS};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;

use futures_lite::{future, io, stream, StreamExt};

use async_io::{block_on, Async};

use blocking::unblock;

mod python;
use python::{wait_signal, PythonIUT};

use termion::{color, style};

async fn connect_to_rootcanal(port: HCI) -> std::io::Result<()> {
    let opts = Opts::from_args();
    let rootcanal_port = opts.rootcanal;
    let tcp = Async::<TcpStream>::connect((Ipv4Addr::LOCALHOST, rootcanal_port))
        .await
        .expect("Connect");

    let (hcirx, hcitx) = io::split(port);
    let (tcprx, tcptx) = io::split(tcp);

    future::or(io::copy(hcirx, tcptx), io::copy(tcprx, hcitx)).await?;

    println!("HCI ended");

    Ok(())
}

#[derive(Debug, Deserialize)]
struct Config {
    ics: HashMap<String, bool>,
    ixit: HashMap<String, HashMap<String, String>>,
    skip: Option<Vec<String>>,
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

    /// All tests under this prefix will be run.
    /// The prefix must include the profile.
    test_prefix: String,

    /// Rootcanal HCI port
    #[structopt(short, long, default_value = "6402")]
    rootcanal: u16,

    /// Selects the Python module implementing PTS interactions
    #[structopt(short, long, default_value = "mmi2grpc")]
    iut: String,

    /// List selected tests and exit
    #[structopt(short, long)]
    list: bool,

    /// IUT parameters
    args: Vec<String>
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
    let mut skip = HashSet::new();

    let profile_name = split_once(&opts.test_prefix, "/")
        .map(|(profile, _)| profile)
        .unwrap_or(&*opts.test_prefix);
    let iut_name: &str = &*opts.iut;
    let iut_args = Arc::new(opts.args.clone());

    if let Some(ref config_path) = opts.config {
        let config_file = File::open(config_path).context("Failed to open config file")?;
        let config: Config = serde_json::from_reader(BufReader::new(config_file))
            .context("Failed to parse config")?;

        for (ics, value) in config.ics {
            pts.set_ics(&*ics, value);
            pts.set_ics(&ics.to_uppercase(), value);
        }

        let pixitx = config.ixit.get("default").context("default IXIT missing")?;
        for (ixit, value) in pixitx {
            pts.set_ixit(&*ixit, &*value);
        }

        let pixitx = config
            .ixit
            .get(profile_name)
            .context("IXIT missing for selected profile")?;
        for (ixit, value) in pixitx {
            pts.set_ixit(&*ixit, &*value);
        }

        for test in config.skip.unwrap_or(vec![]).into_iter() {
            skip.insert(test);
        }
    }

    let profile = Arc::new(
        pts.profile(profile_name)
            .with_context(|| format!("Profile '{}' not found", profile_name))?,
    );

    let tests = profile
        .tests()
        .filter(|test| test.starts_with(&opts.test_prefix))
        .filter(|test| !skip.contains(test))
        .collect::<Vec<_>>();

    println!("Tests: {:?}", tests);
    if opts.list {
        return Ok(());
    }

    block_on(async move {
        let result: Result<Vec<_>> = stream::iter(tests.into_iter())
            .then(|test| {
                let profile = profile.clone();
                let iut_args = iut_args.clone();

                async move {
                    let iut = Arc::new(PythonIUT::new(&iut_name, &iut_args)?);

                    println!("Resetting IUT ...");
                    iut.enter()?;

                    println!("Reading local address ...");
                    let addr = BdAddr::new(
                        iut.address()?
                            .try_into()
                            .map_err(|_| Error::msg("Invalid address size"))?,
                    );

                    println!("Local address: {}", addr);
                    let events = profile
                        .run_test(
                            &*test,
                            addr,
                            connect_to_rootcanal,
                            move |i| {
                                let iut = iut.clone();
                                unblock(move || iut.interact(i))
                            },
                            Some("/tmp/audiodata"),
                        )
                        .await;
                    let signal = unblock(wait_signal);
                    let signal = async move { signal.await.map_err(RunError::Interact) };

                    let verdict = future::or(logger::print(events), signal)
                        .await
                        .context("Runtime Error")?;
                    let verdict = verdict.context("No Verdict ?")?;
                    println!("Verdict: {}", verdict);
                    Ok((test, verdict))
                }
            })
            .try_collect()
            .await;

        report_results(result?);
        Ok(())
    })
}
