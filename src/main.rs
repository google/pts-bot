use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::net::{Ipv4Addr, Shutdown, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::str::from_utf8;
use std::thread;

use anyhow::{Context, Result};
use dirs;
use libpts::{logger, HCIPort, Interaction, IUT, PTS};
use serde::Deserialize;
use serde_json;
use structopt::StructOpt;
use tokio::runtime::Runtime;

mod bluetooth;
mod interact;

use bluetooth::facade::read_only_property_client::ReadOnlyPropertyClient;
use bluetooth::facade::root_facade_client::RootFacadeClient;
use bluetooth::facade::{BluetoothModule, Empty, StartStackRequest, StopStackRequest};

const ROOTCANAL_PORT: u16 = 6402;
const GRPC_PORT: u16 = 8999;
const GRPC_SERVER_ROOT_PORT: u16 = 8998;
const SIGNAL_PORT: u16 = 8997;

use termion::{color, style};

struct Gabeldorsche {
    addr: String,
    rt: Runtime,
    process: Child,
}

impl Gabeldorsche {
    fn spawn(command: &Path) -> Result<Self> {
        let process = Command::new(command)
            .arg(format!("--grpc-port={}", GRPC_PORT))
            .arg(format!("--root-server-port={}", GRPC_SERVER_ROOT_PORT)) // Only for RootFacade service in rootservice.proto
            .arg(format!("--rootcanal-port={}", ROOTCANAL_PORT))
            .arg(format!("--signal-port={}", SIGNAL_PORT))
            .env("ROOTCANAL_PORT", ROOTCANAL_PORT.to_string())
            .spawn()
            .context("Failed to spawn gabeldorsche")?;
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, SIGNAL_PORT))?;
        listener.accept()?;
        let rt = Runtime::new().unwrap();
        let mut addr: String = String::new();
        rt.block_on(async {
            enable_module(BluetoothModule::L2cap).await.unwrap();
            addr = read_local_address().await.unwrap();
        });
        Ok(Self { process, addr, rt })
    }
}

impl Drop for Gabeldorsche {
    fn drop(&mut self) {
        println!("Terminating gabeldorsche");
        self.rt.block_on(async {
            stop_stack_request().await.unwrap();
        });
        let _ = self.process.kill().and_then(|_| self.process.wait());
    }
}

impl IUT for Gabeldorsche {
    fn bd_addr(&self) -> &str {
        &self.addr
    }

    fn interact(&mut self, interaction: Interaction) -> String {
        interact::run(interaction).unwrap()
    }
}

async fn enable_module(
    bluetooth_module: BluetoothModule,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut root_facade_client =
        RootFacadeClient::connect(format!("http://127.0.0.1:{}", GRPC_SERVER_ROOT_PORT)).await?;
    let start_stack_request = tonic::Request::new(StartStackRequest {
        module_under_test: bluetooth_module.into(),
    });
    root_facade_client.start_stack(start_stack_request).await?;
    Ok(())
}

async fn stop_stack_request() -> Result<(), Box<dyn std::error::Error>> {
    let mut root_facade_client =
        RootFacadeClient::connect(format!("http://127.0.0.1:{}", GRPC_SERVER_ROOT_PORT)).await?;
    let stop_stack_request = tonic::Request::new(StopStackRequest {});
    root_facade_client.stop_stack(stop_stack_request).await?;
    Ok(())
}

async fn read_local_address() -> Result<String, Box<dyn std::error::Error>> {
    let mut client = ReadOnlyPropertyClient::connect(format!("http://127.0.0.1:{}", GRPC_PORT))
        .await
        .unwrap();
    let empty = tonic::Request::new(Empty {});
    let addr = match client.read_local_address(empty).await {
        Ok(addr) => from_utf8(&addr.into_inner().address)
            .unwrap()
            .replace(":", "")
            .to_uppercase(),
        Err(err) => {
            println!("Error reading local address: {}", err);
            std::process::exit(2);
        }
    };
    Ok(addr)
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
#[structopt(name = "libpts", about = "libpts gabeldorsche runner")]
struct Opts {
    /// Profile to test, eg "A2DP", "L2CAP", ...
    #[structopt(short, long)]
    profile: String,

    /// Config file path
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,

    /// Gabeldorsche pts binary to use as Implementation Under Test (IUT)
    #[structopt(short, long, parse(from_os_str))]
    gabeldorsche: PathBuf,
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

    let profile = pts
        .profile(&*opts.profile)
        .with_context(|| format!("Profile '{}' not found", &opts.profile))?;

    println!("Tests: {:?}", profile.tests().collect::<Vec<_>>());

    let result = profile
        .tests()
        .map(|test| {
            let mut gabeldorsche = Gabeldorsche::spawn(&opts.gabeldorsche)?;
            let events = profile.run_test(&*test, &mut gabeldorsche);

            let verdict = logger::print(events).context("Runtime Error")?;
            let verdict = verdict.context("No Verdict ?")?;
            println!("Verdict: {}", verdict);
            Ok((test, verdict))
        })
        .collect::<Result<Vec<_>>>()?;

    report_results(result);
    Ok(())
}
