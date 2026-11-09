//! Networking protocol and transport seams for whiteboard peers.

mod config;
/// Deterministic in-process simulation of whiteboard peers.
pub mod sim;
mod transport;

pub use config::MAX_DATAGRAM;
pub use transport::{Chaos, Sim, SimNet, Transport, TransportError, Udp};
