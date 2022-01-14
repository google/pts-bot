use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs::File;
use std::future::Future;
use std::io::BufReader;
use std::net::{Ipv4Addr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::task::Poll;

use anyhow::{Context, Error, Result};
use dirs;
use libpts::{logger, BdAddr, Interaction, HCI, PTS};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;

use futures_lite::{future, io, pin, stream, FutureExt, Stream, StreamExt};

use async_io::{block_on, Async};

use async_ctrlc::CtrlC;

use blocking::unblock;

mod python;
use python::PythonIUT;

mod test;

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

fn abortable<T>(
    mut stream: impl Stream<Item = T> + Unpin,
    mut signal: impl Future<Output = ()> + Unpin,
) -> impl Stream<Item = T> + Unpin {
    stream::poll_fn(move |cx| match stream.poll_next(cx) {
        Poll::Pending => signal.poll(cx).map(|_| None),
        val => val,
    })
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
    args: Vec<String>,
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

    let ctrlc = CtrlC::new().context("Failed to create Ctrl+C handler")?;

    block_on(async move {
        let stream = stream::iter(tests.clone().into_iter()).then(|test| {
            let profile = profile.clone();
            let iut_args = iut_args.clone();

            async move {
                let iut = Arc::new(PythonIUT::new(&iut_name, &iut_args, &*test)?);

                let addr = {
                    let iut = iut.clone();
                    unblock(move || -> Result<BdAddr> {
                        println!("Resetting IUT ...");
                        iut.enter()?;

                        println!("Reading local address ...");
                        Ok(BdAddr::new(
                            iut.address()?
                                .try_into()
                                .map_err(|_| Error::msg("Invalid address size"))?,
                        ))
                    })
                    .await?
                };

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

                let result: Result<test::TestResult> = logger::print(events)
                    .await
                    .context("Runtime Error")
                    .try_into();
                result
            }
        });
        pin!(stream);
        let results: Vec<_> = abortable(stream, ctrlc)
            .chain(stream::repeat_with(|| {
                // Provide a None result to all the test that
                // have not been executed (because of a Ctrl-C)
                Ok(test::TestResult::None)
            }))
            .zip(stream::iter(tests.into_iter()))
            .map(|(result, name)| result.map(|result| test::TestExecution { name, result }))
            .try_collect()
            .await?;

        test::report(results);
        Ok(())
    })
}
