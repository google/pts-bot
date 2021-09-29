use std::convert::AsRef;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

use thiserror::Error;

pub enum WineArch {
    Win32,
    Win64,
}

impl AsRef<str> for WineArch {
    fn as_ref(&self) -> &str {
        match self {
            WineArch::Win32 => "win32",
            WineArch::Win64 => "win64",
        }
    }
}

impl AsRef<OsStr> for WineArch {
    fn as_ref(&self) -> &OsStr {
        AsRef::<str>::as_ref(self).as_ref()
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Prefix creation failed ({0})")]
    Prefix(#[source] io::Error),
    #[error("Server launch failed ({0})")]
    Server(#[source] io::Error),
    #[error("Boot failed ({0})")]
    Boot(#[source] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

struct WineServer(Child);

pub struct Wine {
    server: WineServer,
    prefix: PathBuf,
}

impl Wine {
    pub fn spawn(prefix: PathBuf, arch: WineArch) -> Result<Self> {
        let create_prefix = !prefix.exists();

        if create_prefix {
            fs::create_dir(&prefix)
                .and_then(|_| fs::create_dir(&prefix.join("drive_c")))
                .and_then(|_| fs::create_dir(&prefix.join("dosdevices")))
                .and_then(|_| unix::fs::symlink("../drive_c", &prefix.join("dosdevices/c:")))
                .map_err(|source| {
                    let _ = fs::remove_dir_all(&prefix);
                    Error::Prefix(source)
                })?;
        }

        let server = Command::new("wineserver")
            .arg("--foreground")
            .arg("--persistent")
            .env("WINEPREFIX", &prefix)
            .env("WINEARCH", &arch)
            .spawn()
            .map(WineServer)
            .map_err(Error::Server)?;

        // Wrap the server as soon as possible to drop it properly
        let wine = Wine { server, prefix };

        let metadata = fs::metadata(&wine.prefix).map_err(Error::Prefix)?;

        let directory = format!(
            "/tmp/.wine-{}/server-{:x}-{:x}",
            metadata.uid(),
            metadata.dev(),
            metadata.ino()
        );

        // Wine on Debian is patched to change the wineserver directory
        let debian_directory = format!(
            "/run/user/{}/wine/server-{:x}-{:x}",
            metadata.uid(),
            metadata.dev(),
            metadata.ino()
        );

        let path = Path::new(&directory);
        let debian_path = Path::new(&debian_directory);
        while !path.exists() && !debian_path.exists() {
            thread::sleep(Duration::from_millis(100));
        }

        let status = wine
            .command("wineboot.exe", false)
            .env("WINEARCH", &arch)
            .status()
            .map_err(Error::Boot)?;

        if create_prefix {
            // It seems that the wine prefix is not
            // fully created when wineboot exit
            // so we wait 500ms
            thread::sleep(Duration::from_millis(500));
        }

        if status.success() {
            Ok(wine)
        } else {
            Err(Error::Boot(io::Error::new(
                io::ErrorKind::Other,
                status.code().map_or("exited".to_owned(), |code| {
                    format!("exited with code {}", code)
                }),
            )))
        }
    }

    pub fn drive_c(&self) -> PathBuf {
        self.prefix.join("drive_c")
    }

    pub fn command<S: AsRef<OsStr>>(&self, program: S, with_graphics: bool) -> Command {
        let wine = "wine";
        let mut command = Command::new(if with_graphics { "xvfb-run" } else { wine });
        if with_graphics {
            command.arg(wine);
        }
        command
            .arg(program)
            .env("WINEDLLOVERRIDES", "winedevice.exe=") // Disable device creation
            .env("WINEDEBUG", "-all")
            .env("WINEPREFIX", &self.prefix)
            .env("USER", "pts")
            .current_dir(self.drive_c());
        command
    }

    pub fn devices(&self) -> io::Result<Vec<String>> {
        fs::read_dir(self.prefix.join("dosdevices"))?
            .map(|res| {
                res.and_then(|e| {
                    e.path()
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .ok_or(io::Error::new(io::ErrorKind::Other, "invalid file name"))
                })
            })
            .collect()
    }

    pub fn first_available_com_port(&self) -> io::Result<String> {
        let devices = self.devices()?;

        for n in 1..256 {
            let port = format!("com{}", n);
            if !devices.contains(&port) {
                return Ok(port);
            }
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no availaible com port",
        ))
    }

    pub fn bind_com_port(&self, path: &Path) -> io::Result<String> {
        let port = self.first_available_com_port()?;
        unix::fs::symlink(path, self.prefix.join("dosdevices").join(&port))?;
        Ok(port)
    }

    pub fn unbind_com_port(&self, port: String) -> io::Result<()> {
        fs::remove_file(self.prefix.join("dosdevices").join(port))
    }
}

impl Drop for WineServer {
    fn drop(&mut self) {
        // TODO: handle failure
        let _ = self.0.kill().and_then(|_| self.0.wait());
    }
}
