//! The `Transport` trait and its transports.

mod sim;
mod udp;

pub use sim::{Chaos, Sim, SimNet};
pub use udp::Udp;

use std::net::SocketAddr;
use std::time::Duration;

use crate::config::MAX_DATAGRAM;

/// Moves datagrams between addresses. A datagram arrives whole or not at all; it may also be
/// lost, reordered or duplicated.
pub trait Transport {
    /// Sends `datagram` to `to`.
    ///
    /// # Errors
    ///
    /// [`TransportError::Oversize`] if `datagram` is longer than [`max_datagram`](Self::max_datagram),
    /// and nothing is sent; [`TransportError::Io`] if the socket fails.
    fn send(&mut self, to: SocketAddr, datagram: &[u8]) -> Result<(), TransportError>;

    /// Returns the next datagram that arrived and its sender, waiting up to `timeout` for one;
    /// `None` if none came.
    ///
    /// # Errors
    ///
    /// [`TransportError::Io`] if the socket fails.
    fn recv(&mut self, timeout: Duration) -> Result<Option<(SocketAddr, Vec<u8>)>, TransportError>;

    /// The longest datagram `send` accepts, in bytes.
    fn max_datagram(&self) -> usize;
}

/// Why a datagram could not be sent or received.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// The datagram is longer than the transport sends.
    #[error("a datagram of {len} bytes is longer than the {max}-byte maximum")]
    Oversize {
        /// Length of the rejected datagram.
        len: usize,
        /// The transport's maximum.
        max: usize,
    },
    /// The socket failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

fn check_size(datagram: &[u8]) -> Result<(), TransportError> {
    if datagram.len() > MAX_DATAGRAM {
        return Err(TransportError::Oversize {
            len: datagram.len(),
            max: MAX_DATAGRAM,
        });
    }
    Ok(())
}
