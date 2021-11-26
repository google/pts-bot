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

use thiserror::Error;

pub use crate::bd_addr::BdAddr;
use crate::hci::HCIPort;
use crate::wine::{Wine, WineArch};
use crate::xml_model::{ets::ETS, picsx::PICS, pixitx::PIXIT, XMLModel};

pub use crate::log::Event;
pub use crate::pts::{MMIStyle, Message};

pub struct Interaction<'a> {
    pub style: MMIStyle,
    pub pts_addr: BdAddr,
    pub id: &'a str,
    pub profile: &'a str,
    pub test: &'a str,
    pub description: &'a str,
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
pub enum RunError<E> {
    #[error("IO {0}")]
    IO(#[source] io::Error),
    #[error(transparent)]
    Interact(#[from] E),
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

    pub fn run_test<E: Send + 'static>(
        &self,
        test: &str,
        addr: BdAddr,
        mut interact: impl FnMut(Interaction) -> Result<String, E> + Send + 'static,
        mut pipe_hci: impl FnMut(HCI) -> (),
        audio_output_path: Option<&str>,
    ) -> impl Iterator<Item = Result<Event, RunError<E>>> + 'pts {
        let (port, wineport) = HCIPort::bind(&self.pts.wine).expect("HCI port");
        pipe_hci(port);

        let octet_addr = format!("{:#}", addr);

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

        let pts_addr = messages
            .find_map(|message| match message {
                Ok(Message::Addr { value }) => Some(value),
                _ => None,
            })
            .unwrap();

        let messages = messages.map(move |message| {
            if let Ok(Message::ImplicitSend {
                description: mmi,
                style,
            }) = &message
            {
                let answer = if let Some((raw_id, test, profile, description)) = mmi::parse(mmi) {
                    interact(Interaction {
                        pts_addr,
                        style: *style,
                        id: mmi::id_to_mmi(profile, raw_id).unwrap_or(&raw_id.to_string()),
                        profile,
                        test,
                        description,
                    })
                    .map_err(RunError::Interact)?
                } else {
                    todo!();
                };
                send_answer(&*answer);
            }

            message.map_err(RunError::IO)
        });

        log::parse(messages)
    }
}
