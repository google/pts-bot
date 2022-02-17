mod at;
mod bd_addr;
mod compat;
mod hci;
mod installer;
mod log;
pub mod logger;
mod mmi;
mod pts;
mod ttcn;
mod wine;
mod xml_model;

use std::collections::HashMap;
use std::convert::identity;
use std::io;
use std::path::PathBuf;
use std::task::Poll;
use std::time::Duration;

use futures_lite::{stream, Future, FutureExt, Stream, StreamExt};

use thiserror::Error;

pub use crate::bd_addr::BdAddr;
use crate::hci::HCIPort;
use crate::wine::{Wine, WineArch};
use crate::xml_model::{ets::Ets, picsx::Pics, pixitx::Pixit, XMLModel};

pub use crate::log::Event;
pub use crate::pts::{MMIStyle, Message};

pub struct Interaction {
    pts_addr: BdAddr,
    style: MMIStyle,
    description: String,
}

pub type HCI = HCIPort;

pub struct PTS {
    wine: Wine,
    ics: HashMap<String, bool>,
    ixit: HashMap<String, String>,
}

pub struct Profile<'pts> {
    name: String,
    pts: &'pts PTS,
    ets: Ets,
    pics: Pics,
    pixit: Pixit,
}

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("Wine spawn failed ({0})")]
    Wine(#[source] wine::Error),
    #[error("Server install failed ({0})")]
    Server(#[source] io::Error),
}

#[derive(Debug, Error)]
pub enum RunError<Err1, Err2> {
    #[error("IO error")]
    IO(#[source] io::Error),
    #[error("Pipe HCI failed")]
    Pipe(#[source] Err1),
    #[error("Interact failed")]
    Interact(#[source] Err2),
    #[error("Unable to get PTS Bluetooth Address")]
    NoAddress,
    #[error("Timeout")]
    Timeout,
}

impl Interaction {
    pub fn explode(&self) -> (BdAddr, MMIStyle, &str, &str, &str, &str) {
        if let Some((raw_id, test, profile, description)) = mmi::parse(self.description.as_str()) {
            let id = raw_id
                .parse()
                .ok()
                .and_then(|raw_id| mmi::id_to_mmi(profile, raw_id))
                .unwrap_or(raw_id);
            (self.pts_addr, self.style, id, profile, test, description)
        } else {
            todo!();
        }
    }
}

impl PTS {
    pub fn install(directory: PathBuf, installer: impl io::Read) -> Result<Self, InstallError> {
        let wine = Wine::spawn(directory, WineArch::Win32).map_err(InstallError::Wine)?;

        if installer::is_pts_installation_needed(&wine) {
            installer::install_pts(&wine, installer);
        }

        installer::install_server(&wine).map_err(InstallError::Server)?;

        Ok(Self {
            wine,
            ics: HashMap::new(),
            ixit: HashMap::new(),
        })
    }

    pub fn set_ics(&mut self, name: &str, value: bool) {
        self.ics.insert(name.to_owned(), value);
    }
    pub fn set_ixit(&mut self, name: &str, value: &str) {
        self.ixit.insert(name.to_owned(), value.to_owned());
    }

    pub fn profile(&self, name: &str) -> Option<Profile<'_>> {
        let ets = Ets::parse(name, &self.wine).ok()?;
        let pics = Pics::parse(name, &self.wine).ok()?;
        let pixit = Pixit::parse(name, &self.wine).ok()?;

        Some(Profile {
            pts: self,
            name: name.to_owned(),
            ets,
            pics,
            pixit,
        })
    }
}

impl<'pts> Profile<'pts> {
    pub fn tests(&self) -> impl Iterator<Item = String> + '_ {
        self.ets.enabled_testcases(move |name| {
            self.pts.ics.get(name).copied().or_else(|| {
                self.pics
                    .iter()
                    .find(|row| row.name == name)
                    .map(|row| row.value)
            })
        })
    }

    pub async fn run_test<Fut1, Err1, Fut2, Err2>(
        &self,
        test: &str,
        iut_addr: BdAddr,
        mut pipe_hci: impl FnMut(HCI) -> Fut1 + 'pts,
        mut interact: impl FnMut(Interaction) -> Fut2 + 'pts,
        audio_output_path: Option<&str>,
        inactivity_timeout: u64,
    ) -> impl Stream<Item = Result<Event, RunError<Err1, Err2>>> + 'pts
    where
        Fut1: 'pts + Future<Output = Result<(), Err1>>,
        Err1: 'pts,
        Fut2: 'pts + Future<Output = Result<String, Err2>>,
        Err2: 'pts,
    {
        let (port, wineport) = HCIPort::bind(&self.pts.wine).expect("HCI port");

        let hci = Box::pin(async move {
            pipe_hci(port).await.map_err(RunError::Pipe)?;
            Ok(None)
        });

        let octet_addr = format!("{:#}", iut_addr);

        let pics = self.pics.iter().map(|row| {
            let value = self.pts.ics.get(&row.name).unwrap_or(&row.value);
            let value = if *value { "TRUE" } else { "FALSE" };
            (&*row.name, "BOOLEAN", value)
        });

        let pixit = self.pixit.iter().map(|row| match &*row.name {
            "TSPX_bd_addr_iut" => ("TSPX_bd_addr_iut", "OCTETSTRING", &*octet_addr),
            "TSPX_delete_link_key" => ("TSPX_delete_link_key", "BOOLEAN", "TRUE"),
            _ => {
                let value = self.pts.ixit.get(&row.name).unwrap_or(&row.value);
                (&*row.name, &*row.value_type, &**value)
            }
        });

        let parameters = pics.chain(pixit);

        let (messages, mut send_answer) =
            pts::Server::spawn(wineport, &self.name, test, parameters, audio_output_path)
                .into_parts();

        let mut messages = messages
            .map(|r| r.map_err(RunError::IO))
            .or(stream::once(hci)
                .then(identity)
                .filter_map(Result::transpose));

        let mut pts_addr = messages
            .find_map(|message| match message {
                Ok(Message::Addr { value }) => Some(Ok(value)),
                Err(e) => Some(Err(e)),
                _ => None,
            })
            .await
            .ok_or(RunError::NoAddress)
            .and_then(identity)
            .map_err(|e| stream::once(Err(e)));

        let (tx, rx) = async_channel::unbounded();

        let answers = async move {
            while let Ok(interaction) = rx.recv().await {
                let answer = interact(interaction).await.map_err(RunError::Interact)?;
                send_answer(&*answer);
            }
            Ok(None)
        };

        let mut timeout = async_io::Timer::after(Duration::from_secs(inactivity_timeout));

        let messages = stream::poll_fn(move |cx| match pts_addr {
            Ok(pts_addr) => {
                let message = messages.poll_next(cx);
                if let Poll::Ready(message) = message {
                    timeout.set_after(Duration::from_secs(inactivity_timeout));
                    Poll::Ready(match message {
                        Some(Ok(Message::ImplicitSend { description, style })) => {
                            tx.try_send(Interaction {
                                pts_addr,
                                style,
                                description: description.to_owned(),
                            })
                            .unwrap();
                            Some(Ok(Message::ImplicitSend { description, style }))
                        }
                        None => {
                            tx.close();
                            None
                        }
                        x => x,
                    })
                } else if let Poll::Ready(_) = timeout.poll(cx) {
                    Poll::Ready(Some(Err(RunError::Timeout)))
                } else {
                    Poll::Pending
                }
            }
            Err(ref mut e) => e.poll_next(cx),
        })
        .or(stream::once(answers)
            .then(identity)
            .filter_map(Result::transpose));

        log::parse(messages)
    }
}
