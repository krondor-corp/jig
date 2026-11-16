//! The TCP listener both IP backends share: bound lazily, filtered by caller.

use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::sync::Mutex;

use socket2::{Domain, Protocol, Socket, Type};

use super::{NetError, NodeId, Stream, DIAL_TIMEOUT};

/// A listener that does not exist until something asks to accept on it.
#[derive(Default)]
pub(super) struct LazyListener {
    bound: Mutex<Option<TcpListener>>,
}

impl LazyListener {
    /// Bind to `addr()` if not yet bound; return a handle and its address.
    pub(super) fn get(
        &self,
        addr: impl FnOnce() -> Result<SocketAddr, NetError>,
    ) -> Result<(TcpListener, SocketAddr), NetError> {
        let mut bound = self.bound.lock().unwrap_or_else(|e| e.into_inner());
        if bound.is_none() {
            *bound = Some(TcpListener::bind(addr()?)?);
        }
        let listener = bound.as_ref().expect("bound above");
        Ok((listener.try_clone()?, listener.local_addr()?))
    }

    /// Accept until a caller passes `identify`, dropping everyone else.
    ///
    /// `identify` returns `Ok(None)` for an untrusted caller. An error from it
    /// (say, `tailscale whois` failing) drops that connection too: a caller
    /// whose identity cannot be established is not trusted.
    pub(super) fn accept(
        &self,
        addr: impl FnOnce() -> Result<SocketAddr, NetError>,
        identify: impl Fn(IpAddr) -> Result<Option<NodeId>, NetError>,
    ) -> Result<(Box<dyn Stream>, NodeId), NetError> {
        let (listener, _) = self.get(addr)?;
        loop {
            let (stream, from) = listener.accept()?;
            match identify(from.ip()) {
                Ok(Some(id)) => return Ok((Box::new(stream), id)),
                Ok(None) => tracing::warn!(%from, "refused connection from untrusted caller"),
                Err(e) => tracing::warn!(%from, "refused connection, identity unknown: {e}"),
            }
        }
    }
}

pub(super) fn connect(addr: SocketAddr) -> Result<Box<dyn Stream>, NetError> {
    Ok(Box::new(TcpStream::connect_timeout(&addr, DIAL_TIMEOUT)?))
}

/// Connect to `addr` from `source`, this machine's own address on the
/// network, when identity is the caller's address.
///
/// Left to itself the OS picks the source address, and not necessarily
/// ours: with several addresses on one interface (a host on two overlays,
/// or loopback aliases, where macOS sends from the destination's own
/// address) the peer would see an address it does not trust.
pub(super) fn connect_from(source: IpAddr, addr: SocketAddr) -> Result<Box<dyn Stream>, NetError> {
    let socket = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))?;
    socket.bind(&SocketAddr::new(source, 0).into())?;
    socket.connect_timeout(&addr.into(), DIAL_TIMEOUT)?;
    Ok(Box::new(TcpStream::from(socket)))
}
