//! The real transport: a UDP socket.

use std::io::ErrorKind;
use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::time::Duration;

use super::{Transport, TransportError, check_size};
use crate::config::MAX_DATAGRAM;

/// Fits the largest UDP payload, so a received datagram is never cut short.
const RECV_BUFFER: usize = u16::MAX as usize;

/// A UDP socket.
#[derive(Debug)]
pub struct Udp {
    socket: UdpSocket,
    buffer: Box<[u8]>,
}

impl Udp {
    /// Binds a socket to `addr`; port 0 lets the OS pick a free one.
    ///
    /// # Errors
    ///
    /// [`TransportError::Io`] if the address cannot be bound.
    pub fn bind(addr: impl ToSocketAddrs) -> Result<Self, TransportError> {
        Ok(Self {
            socket: UdpSocket::bind(addr)?,
            buffer: vec![0; RECV_BUFFER].into_boxed_slice(),
        })
    }

    /// The address the socket is bound to.
    ///
    /// # Errors
    ///
    /// [`TransportError::Io`] if the OS cannot tell.
    pub fn local_addr(&self) -> Result<SocketAddr, TransportError> {
        Ok(self.socket.local_addr()?)
    }
}

impl Transport for Udp {
    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), TransportError> {
        check_size(datagram)?;
        match self.socket.send_to(datagram, to) {
            // After a zero-timeout `recv` the socket is non-blocking, and a full send buffer
            // drops the datagram, as the network may anyway.
            Err(error) if error.kind() == ErrorKind::WouldBlock => Ok(()),
            result => result.map(drop).map_err(Into::into),
        }
    }

    /// A zero `timeout` returns at once.
    fn recv(&mut self, timeout: Duration) -> Result<Option<(SocketAddr, Vec<u8>)>, TransportError> {
        // `set_read_timeout` rejects zero, so not waiting needs non-blocking mode.
        self.socket.set_nonblocking(timeout.is_zero())?;
        if !timeout.is_zero() {
            self.socket.set_read_timeout(Some(timeout))?;
        }
        match self.socket.recv_from(&mut self.buffer) {
            Ok((len, from)) => Ok(Some((from, self.buffer[..len].to_vec()))),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                Ok(None)
            }
            Err(error) => Err(error.into()),
        }
    }

    fn max_datagram(&self) -> usize {
        MAX_DATAGRAM
    }
}
