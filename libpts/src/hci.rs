use std::io;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use nix;
use nix::fcntl::OFlag;
use nix::pty;

use async_io::Async;

use futures_lite::{ready, AsyncRead, AsyncWrite};

use crate::wine::Wine;

fn nix_error_into_io_error(error: nix::Error) -> io::Error {
    match error {
        nix::Error::Sys(errno) => errno.into(),
        nix::Error::InvalidPath => io::Error::new(io::ErrorKind::InvalidData, error),
        nix::Error::InvalidUtf8 => io::Error::new(io::ErrorKind::InvalidData, error),
        // FIXME: Change Other to Unsupported when gLinux rustc version >= 1.53.0
        nix::Error::UnsupportedOperation => io::Error::new(io::ErrorKind::Other, error),
    }
}

pub struct HCIPort {
    pty: Async<pty::PtyMaster>,
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
                pty: Async::new(pty)?,
                waiting_read: true,
            },
            WineHCIPort {
                com: Some(com),
                wine,
            },
        ))
    }
}

impl AsyncRead for HCIPort {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match ready!(Pin::new(&mut self.pty).poll_read(cx, buf)) {
            Ok(read) => {
                // Read was successful, something is connected
                self.waiting_read = false;
                Poll::Ready(Ok(read))
            }
            Err(err) if err.kind() == io::Error::from(nix::errno::Errno::EIO).kind() => {
                // The pty will not be connected directly,
                // we want to wait for a connection, but when it disconnect
                // we want to end the read
                Poll::Ready(if self.waiting_read {
                    Err(io::Error::new(io::ErrorKind::Interrupted, "Not connected"))
                } else {
                    Ok(0)
                })
            }
            Err(err) => Poll::Ready(Err(err)),
        }
    }
}

impl AsyncWrite for HCIPort {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let pty: &mut Async<pty::PtyMaster> = &mut self.pty;
        Pin::new(pty).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let pty: &mut Async<pty::PtyMaster> = &mut self.pty;
        Pin::new(pty).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let pty: &mut Async<pty::PtyMaster> = &mut self.pty;
        Pin::new(pty).poll_close(cx)
    }
}

impl<'a> std::ops::Drop for WineHCIPort<'a> {
    fn drop(&mut self) {
        if let Some(com) = self.com.take() {
            let _ = self.wine.unbind_com_port(com);
        }
    }
}
