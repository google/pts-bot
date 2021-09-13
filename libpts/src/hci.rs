use std::io;
use std::os::unix::io::AsRawFd;
use std::path::Path;
use std::sync::Arc;

use nix;
use nix::fcntl::OFlag;
use nix::pty;
use nix::unistd;

use crate::wine::Wine;

fn nix_error_into_io_error(error: nix::Error) -> io::Error {
    match error {
        nix::Error::Sys(errno) => errno.into(),
        nix::Error::InvalidPath => io::Error::new(io::ErrorKind::InvalidData, error),
        nix::Error::InvalidUtf8 => io::Error::new(io::ErrorKind::InvalidData, error),
        nix::Error::UnsupportedOperation => io::Error::new(io::ErrorKind::Unsupported, error),
    }
}

#[derive(Clone)]
pub struct HCIPort {
    pty: Arc<pty::PtyMaster>,
    waiting_read: bool,
}

pub struct WineHCIPort<'wine> {
    pub(crate) wine: &'wine Wine,
    pub(crate) com: Option<String>,
}

impl<'a> HCIPort {
    pub fn bind(wine: &'a Wine) -> io::Result<(HCIPort, WineHCIPort<'a>)> {
        let pty =
            pty::posix_openpt(OFlag::O_RDWR | OFlag::O_NOCTTY).map_err(nix_error_into_io_error)?;

        pty::grantpt(&pty).map_err(nix_error_into_io_error)?;
        pty::unlockpt(&pty).map_err(nix_error_into_io_error)?;

        let path = pty::ptsname_r(&pty).map_err(nix_error_into_io_error)?;

        let com = wine.bind_com_port(Path::new(&path))?;

        Ok((
            HCIPort {
                pty: Arc::new(pty),
                waiting_read: true,
            },
            WineHCIPort {
                com: Some(com),
                wine,
            },
        ))
    }
}

impl io::Read for HCIPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match unistd::read(self.pty.as_raw_fd(), buf) {
            Ok(read) => {
                // Read was sucessfull, something is connected
                self.waiting_read = false;
                Ok(read)
            }
            Err(nix::Error::Sys(nix::errno::Errno::EIO)) => {
                // The pty will not be connected directly,
                // we want to wait for a connection, but when it disconnect
                // we want to end the read
                if self.waiting_read {
                    Err(io::Error::new(io::ErrorKind::Interrupted, "Not connected"))
                } else {
                    Ok(0)
                }
            }
            Err(err) => Err(nix_error_into_io_error(err)),
        }
    }
}

impl io::Write for HCIPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unistd::write(self.pty.as_raw_fd(), buf).map_err(nix_error_into_io_error)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> std::ops::Drop for WineHCIPort<'a> {
    fn drop(&mut self) {
        if let Some(com) = self.com.take() {
            let _ = self.wine.unbind_com_port(com);
        }
    }
}
