use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::net::{Ipv4Addr, TcpStream};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Child, ChildStderr, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::task::Poll;

use anyhow::{Context, Result};
use dirs;
use libpts::{logger, BdAddr, Interaction, MMIStyle, PTS};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;
use termion::{color, style};

use async_io::{block_on, Async};
use futures_lite::{
    io::BufReader, io::Lines, ready, AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncWrite, Future,
    StreamExt,
};

const ROOTCANAL_PORT: u16 = 6402;

struct Eiffel {
    process: Child,
    addr: BdAddr,
    lines: Lines<BufReader<Async<ChildStderr>>>,
    stdin: ChildStdin,
}

impl Eiffel {
    async fn spawn(command: &Path, test: &String) -> Result<Self> {
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

        let stderr = process.stderr.take().unwrap();
        let stdin = process.stdin.take().unwrap();

        let mut lines = futures_lite::io::BufReader::new(Async::new(stderr).unwrap()).lines();
        let addr = lines.next().await.unwrap().unwrap().parse().unwrap();

        Ok(Self {
            process,
            addr,
            lines,
            stdin,
        })
    }

    async fn interact(&mut self, interaction: Interaction) -> io::Result<String> {
        let (addr, style, id, test, _, description) = interaction.explode();
        let values = match style {
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
            addr = addr,
            id = id,
            test = test,
            values = values,
            description = description
        )
        .unwrap();

        self.stdin.flush().unwrap();
        self.lines.next().await.unwrap()
    }
}

impl Drop for Eiffel {
    fn drop(&mut self) {
        println!("Terminating eiffel");
        let _ = self.process.kill().and_then(|_| self.process.wait());
    }
}

fn poll_copy<R, W>(
    r: &mut R,
    w: &mut W,
    cx: &mut std::task::Context<'_>,
) -> Poll<std::io::Result<bool>>
where
    R: AsyncBufRead + AsyncWrite + Unpin,
    W: AsyncBufRead + AsyncWrite + Unpin,
{
    let buffer = ready!(Pin::new(&mut *r).poll_fill_buf(cx))?;
    if buffer.is_empty() {
        ready!(Pin::new(&mut *r).poll_flush(cx))?;
        ready!(Pin::new(&mut *w).poll_flush(cx))?;
        return Poll::Ready(Ok(true));
    }

    let i = ready!(Pin::new(&mut *w).poll_write(cx, buffer))?;
    if i == 0 {
        return Poll::Ready(Err(std::io::ErrorKind::WriteZero.into()));
    }

    r.consume(i);
    Poll::Ready(Ok(false))
}

pub async fn copy_bidirectional<A, B>(a: &mut A, b: &mut B) -> std::io::Result<()>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    struct CopyFuture<A, B> {
        a: A,
        b: B,
    }

    impl<A, B> Future for CopyFuture<A, B>
    where
        A: AsyncBufRead + AsyncWrite + Unpin,
        B: AsyncBufRead + AsyncWrite + Unpin,
    {
        type Output = std::io::Result<()>;

        fn poll(mut self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
            let CopyFuture { a, b } = &mut *self;
            loop {
                let a_to_b = poll_copy(&mut *a, &mut *b, cx);
                let b_to_a = poll_copy(&mut *b, &mut *a, cx);
                if ready!(a_to_b)? || ready!(b_to_a)? {
                    return Poll::Ready(Ok(()));
                }
            }
        }
    }

    CopyFuture {
        a: BufReader::new(a),
        b: BufReader::new(b),
    }
    .await
}

#[derive(Debug, Deserialize)]
struct Config {
    ics: HashMap<String, bool>,
    ixit: HashMap<String, HashMap<String, String>>,
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

    /// Filter tests to execute
    #[structopt(short, long)]
    test: Option<String>,
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

        let pixitx = config.ixit.get("default").context("default IXIT missing")?;
        for (ixit, value) in pixitx {
            pts.set_ixit(&*ixit, &*value);
        }

        let pixitx = config
            .ixit
            .get(&opts.profile)
            .context("IXIT missing for selected profile")?;
        for (ixit, value) in pixitx {
            pts.set_ixit(&*ixit, &*value);
        }
    }

    let profile = pts
        .profile(&*opts.profile)
        .with_context(|| format!("Profile '{}' not found", &opts.profile))?;

    println!("Tests: {:?}", profile.tests().collect::<Vec<_>>());

    let results = profile
        .tests()
        .filter(|test| opts.test.is_none() || opts.test.as_ref() == Some(test))
        .map(|test| {
            let mut eiffel = block_on(Eiffel::spawn(&opts.eiffel, &test))?;
            let addr = eiffel.addr;

            let events = block_on(profile.run_test(
                &*test,
                addr,
                |mut port| async move {
                    let tcp =
                        TcpStream::connect((Ipv4Addr::LOCALHOST, ROOTCANAL_PORT)).expect("Connect");
                    let mut tcp = Async::new(tcp)?;

                    copy_bidirectional(&mut tcp, &mut port).await
                },
                move |i| Box::new(eiffel.interact(i)),
                None,
            ));

            let verdict = block_on(logger::print(events)).context("Runtime Error")?;
            let verdict = verdict.context("No Verdict ?")?;

            println!("Verdict: {}", verdict);
            Ok((test, verdict))
        })
        .collect::<Result<Vec<_>>>()?;

    report_results(results);

    Ok(())
}
