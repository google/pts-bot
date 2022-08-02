use std::convert::AsRef;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::os::unix;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;

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
    _server: WineServer,
    prefix: PathBuf,
}

const EMPTY_FONTCONFIG_FILE: &str = r#"<?xml version="1.0"?>
<!DOCTYPE fontconfig SYSTEM "fonts.dtd">
<fontconfig>
</fontconfig>"#;

const ALSA_CONFIG_FILE: &str = r#"pcm.!default {
    type file
    slave {
        pcm {
            type null
        }
    }
    file {
        @func getenv
        vars [ ALSA_OUTPUT_FILE ]
        default "/dev/null"
    }
    format "wav"
}"#;

impl Wine {
    pub fn spawn(prefix: PathBuf, arch: WineArch) -> Result<Self> {
        let create_prefix = !prefix.exists();

        if create_prefix {
            fs::create_dir_all(&prefix)
                .and_then(|_| fs::create_dir(&prefix.join("drive_c")))
                .and_then(|_| fs::create_dir(&prefix.join("dosdevices")))
                .and_then(|_| unix::fs::symlink("../drive_c", &prefix.join("dosdevices/c:")))
                // See command function
                .and_then(|_| fs::write(&prefix.join("fonts.conf"), EMPTY_FONTCONFIG_FILE))
                .and_then(|_| fs::write(&prefix.join("alsa.conf"), ALSA_CONFIG_FILE))
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
            // Do no inherit stderr as wineserver will fork itself
            // and create daemon processes but will only set stdin
            // and stdout of thoses daemons. This result in the
            // stderr of the parent to be kept and could live more
            // than the parent process resulting in the parent
            // stderr to be never closed
            .stderr(Stdio::null())
            .spawn()
            .map(WineServer)
            .map_err(Error::Server)?;

        // Wrap the server as soon as possible to drop it properly
        let wine = Wine {
            _server: server,
            prefix,
        };

        let metadata = fs::metadata(&wine.prefix).map_err(Error::Prefix)?;

        let directory = format!(
            "/tmp/.wine-{}/server-{:x}-{:x}",
            metadata.uid(),
            metadata.dev(),
            metadata.ino()
        );

        // Wine on Debian is patched to change the wineserver directory
        // /!\ If this repository does not exist yet, wineserver will default
        // to using a randomly generated directory with the path
        // '/tmp/wine-${random}/'.
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

        // The installer can fail if wineboot.exe is not executed
        // with an X display.
        let status = wine
            .command("wineboot.exe", true, None)
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

    pub fn command(
        &self,
        program: &str,
        with_graphics: bool,
        audio_output_path: Option<&str>,
    ) -> Command {
        let mut command = if with_graphics {
            let mut command = Command::new("xvfb-run");
            command.arg("--auto-servernum");
            command.arg("wine");
            command
        } else {
            Command::new("wine")
        };

        command
            .arg(program)
            // winedevice.exe automaticaly create devices under the
            // dosdevices folder, we don't want that, because we are
            // creating them ourselves via bind_com_port so we disable
            // it by preventing wine from loading it
            //
            // winepulse.drv is one of the two audio driver implementation
            // of wine, the other one being winealsa.drv, we prevent
            // winepulse.drv from loading to always have winealsa.drv
            // as audio driver
            .env("WINEDLLOVERRIDES", "winedevice.exe=;winepulse.drv=")
            // On gLinux on cloudtop the cups print server is
            // not accessible. This adds 20 seconds to wine startup
            // waiting for the connection to the server to timeout
            // The PTS don't need printers, so we disable the default
            // cups config
            .env("CUPS_SERVERROOT", "/dev/null")
            // On a system with a lot of fonts (like gLinux), wine can
            // take some time to process them (~8 seconds on gLinux)
            // we don't "render" anything so we provide a fontconfig
            // file without any font
            .env("FONTCONFIG_FILE", &self.prefix.join("fonts.conf"))
            // Controll the alsa configuration to be able to provide
            // a device in a headless environment (the PTS need to play
            // audio for the tester to verify it) and also to be
            // able to save audio to a file
            .env("ALSA_CONFIG_PATH", &self.prefix.join("alsa.conf"))
            .env("WINEDEBUG", "-all")
            .env("WINEPREFIX", &self.prefix)
            .env("USER", "pts")
            .current_dir(self.drive_c());

        if let Some(audio_output_path) = audio_output_path {
            command.env("ALSA_OUTPUT_FILE", audio_output_path);
        }

        command
    }

    pub fn devices(&self) -> io::Result<Vec<String>> {
        fs::read_dir(self.prefix.join("dosdevices"))?
            .map(|res| {
                res.and_then(|e| {
                    e.path()
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "invalid file name"))
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
        let pid = Pid::from_raw(self.0.id() as i32);
        let _ = signal::kill(pid, Signal::SIGINT)
            .map_err(io::Error::from)
            .and_then(|_| self.0.wait());
    }
}
