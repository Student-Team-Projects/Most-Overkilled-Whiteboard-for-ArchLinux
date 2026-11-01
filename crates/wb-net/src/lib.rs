//! Networking protocol and transport seams for whiteboard peers.

pub mod config;
mod envelope;
mod inbox;
/// Deterministic in-process simulation of whiteboard peers.
pub mod sim;

pub use envelope::{BoardMsg, Body, EncodeError, Packet, Reason, Reject};
pub use inbox::Inbox;
