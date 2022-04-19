use std::io;
use std::path::Path;
use std::pin::Pin;
use std::task::{Context, Poll};

use nix::fcntl::OFlag;
use nix::pty;

use async_io::Async;

use futures_lite::{ready, AsyncRead, AsyncWrite};

use crate::wine::Wine;

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
        let pty = pty::posix_openpt(OFlag::O_RDWR | OFlag::O_NOCTTY)?;

        pty::grantpt(&pty)?;
        pty::unlockpt(&pty)?;

        let path = pty::ptsname_r(&pty)?;

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
        loop {
            match ready!(Pin::new(&mut self.pty).poll_read(cx, buf)) {
                Ok(read) => {
                    // Read was successful, something is connected
                    self.waiting_read = false;
                    return Poll::Ready(Ok(read));
                }
                Err(err) if err.kind() == io::Error::from(nix::errno::Errno::EIO).kind() => {
                    // The pty will not be connected directly,
                    // we want to wait for a connection, but when it disconnect
                    // we want to end the read.
                    if !self.waiting_read {
                        return Poll::Ready(Ok(0));
                    }
                }
                Err(err) => return Poll::Ready(Err(err)),
            }
            ready!(Pin::new(&mut self.pty).poll_readable(cx))?;
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
