use std::time::Duration;
use std::collections::{HashMap, HashSet};
use std::convert::TryInto;
use std::fs::File;
use std::future::Future;
use std::io::{stdout, BufReader};
use std::net::{Ipv4Addr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::task::Poll;

use anyhow::{Context, Error, Result};
use dirs;
use libpts::{final_verdict, logger, map_with_stack, BdAddr, Interaction, HCI, PTS};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;

use futures_lite::{future, io, pin, ready, stream, FutureExt, Stream, StreamExt};

use async_io::{block_on, Async};

use async_ctrlc::CtrlC;

use blocking::unblock;

mod python;
use python::PythonIUT;

mod test;

async fn connect_to_hci(port: HCI) -> std::io::Result<()> {
    let opts = Opts::from_args();
    let hci_port = opts.hci;
    let tcp = Async::<TcpStream>::connect((Ipv4Addr::LOCALHOST, hci_port)).await?;

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

fn take_until<T>(
    mut stream: impl Stream<Item = T> + Unpin,
    mut predicate: impl FnMut(&T) -> bool,
) -> impl Stream<Item = T> + Unpin {
    let mut flag = false;
    stream::poll_fn(move |cx| {
        if flag {
            Poll::Ready(None)
        } else {
            Poll::Ready(if let Some(x) = ready!(stream.poll_next(cx)) {
                if predicate(&x) {
                    flag = true;
                }
                Some(x)
            } else {
                None
            })
        }
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

    /// HCI port
    #[structopt(short, long, default_value = "6402")]
    hci: u16,

    /// Selects the Python module implementing PTS interactions
    #[structopt(short, long, default_value = "mmi2grpc")]
    iut: String,

    /// List selected tests and exit
    #[structopt(short, long)]
    list: bool,

    /// Stop after first non sucessfull result
    #[structopt(long)]
    fail_fast: bool,

    /// Test inactivity timeout
    #[structopt(short = "t", long, default_value = "60")]
    inactivity_timeout: u64,

    /// PTS setup executable Path
    #[structopt(long, parse(from_os_str))]
    pts_setup: Option<PathBuf>,

    /// All tests under this prefix will be run.
    /// The prefix must include the profile.
    test_prefix: String,

    /// IUT parameters
    args: Vec<String>,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();

    let installer = opts
        .pts_setup
        .as_ref()
        .map(|path| File::open(path).context("Installer not found"))
        .unwrap_or_else(|| {
            // Load default from config dir
            let mut config = dirs::config_dir().context("Failed to get config dir")?;
            config.push("pts");

            File::open(config.join("pts_setup_8_0_3.exe")).with_context(|| {
                format!(
                    "Installer (pts_setup_8_0_3.exe) not found in {}, {}",
                    config.display(),
                    "download it from the SIG website and add it",
                )
            })
        })?;

    let mut cache = dirs::cache_dir().context("Failed to get cache dir")?;
    cache.push("pts");

    let mut pts = PTS::install(cache, installer).context("Failed to create PTS")?;
    let mut skip = HashSet::new();

    let profile_name = opts
        .test_prefix
        .split_once("/")
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
    let fail_fast = opts.fail_fast;
    let inactivity_timeout = opts.inactivity_timeout;

    block_on(async move {
        let stream = stream::iter(tests.clone().into_iter()).then(|test| {
            let profile = profile.clone();
            let iut_args = iut_args.clone();

            async move {
                let iut = Arc::new(PythonIUT::new(&iut_name, &iut_args, &*test)?);
                let timeout = async_io::Timer::after(Duration::from_secs(inactivity_timeout));

                let addr = {
                    let iut = iut.clone();

                    future::or(
                        unblock(move || -> Result<BdAddr> {
                            println!("Resetting IUT ...");
                            iut.enter()?;

                            println!("Reading local address ...");
                            Ok(BdAddr::new(
                                iut.address()?
                                    .try_into()
                                    .map_err(|_| Error::msg("Invalid address size"))?,
                            ))
                        }),
                        async {
                            timeout.await;
                            anyhow::bail!("Timeout in IUT initialization")
                        },
                    )
                    .await?
                };

                println!("Local address: {}", addr);
                let events = profile
                    .run_test(
                        &*test,
                        addr,
                        connect_to_hci,
                        move |i| {
                            let iut = iut.clone();
                            unblock(move || iut.interact(i))
                        },
                        Some("/tmp/audiodata"),
                        inactivity_timeout,
                    )
                    .await;

                let events = map_with_stack(events, |result| {
                    result.map(|(event, stack)| {
                        logger::print(&mut stdout(), &event, stack).unwrap();
                        event
                    })
                });

                let result: Result<test::TestResult> = final_verdict(events)
                    .await
                    .context("Runtime Error")
                    .try_into();
                result
            }
        });
        pin!(stream);
        let results: Vec<_> = take_until(abortable(stream, ctrlc), |result| {
            fail_fast && !matches!(result, Ok(test::TestResult::Pass))
        })
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
