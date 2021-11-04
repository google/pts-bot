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

use std::cell::Cell;
use std::collections::HashMap;
use std::io;
use std::ops::Fn;
use std::path::PathBuf;
use std::rc::Rc;

use thiserror::Error;

pub use crate::bd_addr::BdAddr;
use crate::hci::WineHCIPort;
use crate::wine::{Wine, WineArch};
use crate::xml_model::{ets::ETS, picsx::PICS, pixitx::PIXIT, XMLModel};

pub use crate::hci::HCIPort;
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

/// Implementation Under Test
pub trait IUT {
    fn bd_addr(&self) -> &str;

    fn interact(&mut self, interaction: Interaction) -> String;
}

pub trait HCI: Sized {
    fn pipe(&self, port: HCIPort);
}

impl<T> HCI for T
where
    T: Fn(HCIPort) -> (),
{
    fn pipe(&self, port: HCIPort) {
        self(port);
    }
}

pub struct PTS<T> {
    wine: Wine,
    ics: HashMap<String, bool>,
    ixit: HashMap<String, String>,
    hci: T,
}

pub struct Profile<'pts, T> {
    name: String,
    pts: &'pts PTS<T>,
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

impl<T> PTS<T>
where
    T: HCI,
{
    pub fn install(
        directory: PathBuf,
        hci: T,
        installer: impl io::Read,
    ) -> Result<Self, InstallError> {
        let wine = Wine::spawn(directory, WineArch::Win32).map_err(InstallError::Wine)?;

        if installer::is_pts_installation_needed(&wine) {
            installer::install_pts(&wine, installer);
        }

        installer::install_server(&wine).map_err(InstallError::Server)?;

        Ok(Self {
            wine,
            hci,
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

    pub fn profile(&self, name: &str) -> Option<Profile<'_, T>> {
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

    fn open_hci(&self) -> WineHCIPort {
        let (port, wineport) = HCIPort::bind(&self.wine).expect("HCI port");

        self.hci.pipe(port);

        wineport
    }
}

impl<'pts, T> Profile<'pts, T>
where
    T: HCI,
{
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

    pub fn run_test<I: IUT>(
        &self,
        test: &str,
        iut: &'pts mut I,
    ) -> impl Iterator<Item = Result<Event, io::Error>> + 'pts {
        let addr = Rc::new(Cell::new(BdAddr::NULL));
        let addr_ptr = addr.clone();

        let wineport = self.pts.open_hci();

        let bd_addr = iut.bd_addr().to_owned();

        let pics = self.pics.iter().map(|row| {
            let value = self.pts.ics.get(&row.name).unwrap_or(&row.value);
            let value = if *value { "TRUE" } else { "FALSE" };
            (&*row.name, "BOOLEAN", value)
        });
        let pixit = self.pixit.iter().map(|row| match &*row.name {
            "TSPX_bd_addr_iut" => ("TSPX_bd_addr_iut", "OCTETSTRING", &*bd_addr),
            "TSPX_delete_link_key" => ("TSPX_delete_link_key", "BOOLEAN", "TRUE"),
            _ => {
                let value = self.pts.ixit.get(&row.name).unwrap_or(&row.value);
                (&*row.name, &*row.value_type, &**value)
            }
        });

        let parameters = pics.chain(pixit);

        let messages = pts::run(wineport, &self.name, test, parameters, move |mmi, style| {
            if let Some((raw_id, test, profile, description)) = mmi::parse(mmi) {
                iut.interact(Interaction {
                    pts_addr: addr.get(),
                    style,
                    id: mmi::id_to_mmi(profile, raw_id).unwrap_or(&raw_id.to_string()),
                    profile,
                    test,
                    description,
                })
            } else {
                todo!();
            }
        });

        let messages = messages.inspect(move |message| {
            if let Ok(Message::Addr { value }) = message {
                addr_ptr.set(*value);
            }
        });

        log::parse(messages)
    }
}
