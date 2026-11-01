//! Named test parameters.

/// Generated stroke points and sync payloads have at most this many bytes, so every generated
/// packet fits in one datagram.
pub const MAX_PAYLOAD: usize = 64;

/// Random datagrams are up to this many bytes: past `MAX_DATAGRAM`, as a hostile sender may send.
pub const MAX_RANDOM: usize = 2 * wb_net::config::MAX_DATAGRAM;
