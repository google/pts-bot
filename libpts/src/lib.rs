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
use std::io;
use std::path::PathBuf;

use std::pin::Pin;
use std::task::Poll;

use futures_lite::{ready, stream::poll_fn, Future, Stream};

use thiserror::Error;

pub use crate::bd_addr::BdAddr;
use crate::hci::HCIPort;
use crate::wine::{Wine, WineArch};
use crate::xml_model::{ets::ETS, picsx::PICS, pixitx::PIXIT, XMLModel};

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
    ets: ETS,
    pics: PICS,
    pixit: PIXIT,
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
    #[error("IO error ({0})")]
    IO(#[source] io::Error),
    #[error("Pipe HCI failed ({0})")]
    Pipe(#[source] Err1),
    #[error("Interact failed ({0})")]
    Interact(#[source] Err2),
    #[error("HCI interrupted")]
    HCIInterrupted,
}

impl Interaction {
    pub fn explode(&self) -> (BdAddr, MMIStyle, String, &str, &str, &str) {
        if let Some((raw_id, test, profile, description)) = mmi::parse(self.description.as_str()) {
            let id = mmi::id_to_mmi(profile, raw_id)
                .map(|x| x.to_string())
                .unwrap_or(raw_id.to_string());
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
        let ets = ETS::parse(name, &self.wine).ok()?;
        let pics = PICS::parse(name, &self.wine).ok()?;
        let pixit = PIXIT::parse(name, &self.wine).ok()?;

        Some(Profile {
            pts: &self,
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

    pub fn run_test<Fut1, Err1, Fut2, Err2>(
        &self,
        test: &str,
        iut_addr: BdAddr,
        mut pipe_hci: impl FnMut(HCI) -> Fut1,
        mut interact: impl FnMut(Interaction) -> Fut2 + 'pts,
        audio_output_path: Option<&str>,
    ) -> impl Stream<Item = Result<Event, RunError<Err1, Err2>>> + 'pts
    where
        Fut1: 'pts + Future<Output = Result<(), Err1>>,
        Err1: 'pts,
        Fut2: 'pts + Future<Output = Result<String, Err2>>,
        Err2: 'pts,
    {
        let (port, wineport) = HCIPort::bind(&self.pts.wine).expect("HCI port");
        let mut hci = pipe_hci(port);

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

        let (mut messages, mut send_answer) =
            pts::Server::spawn(wineport, &self.name, test, parameters, audio_output_path)
                .into_parts();

        let mut pts_addr = BdAddr::NULL;
        let mut answer: Option<Fut2> = None;

        log::parse(poll_fn(move |cx| {
            match unsafe { Pin::new_unchecked(&mut hci) }.poll(cx) {
                Poll::Pending => {}
                Poll::Ready(Ok(_)) => return Poll::Ready(Some(Err(RunError::HCIInterrupted))),
                Poll::Ready(Err(err)) => {
                    return Poll::Ready(Some(Err(err).map_err(RunError::Pipe)))
                }
            }

            if let Some(mut fut) = answer.take() {
                match unsafe { Pin::new_unchecked(&mut fut) }.poll(cx) {
                    Poll::Pending => {
                        answer = Some(fut);
                    }
                    Poll::Ready(Ok(s)) => {
                        send_answer(s.as_str());
                        answer = None;
                    }
                    Poll::Ready(Err(err)) => {
                        return Poll::Ready(Some(Err(err).map_err(RunError::Interact)))
                    }
                }
            }

            let message = ready!(unsafe { Pin::new_unchecked(&mut messages) }.poll_next(cx));
            match &message {
                Some(Ok(Message::Addr { value })) => pts_addr = *value,
                Some(Ok(Message::ImplicitSend {
                    description: mmi,
                    style,
                })) => {
                    answer = Some(interact(Interaction {
                        pts_addr,
                        style: *style,
                        description: mmi.clone(),
                    }))
                }
                _ => {}
            }
            Poll::Ready(message.map(|x| x.map_err(RunError::IO)))
        }))
    }
}
