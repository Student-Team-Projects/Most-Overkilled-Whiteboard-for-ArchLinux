//! Named test parameters.

use wb_core::BoardId;

/// The board the inbox under test is on.
pub const OURS: BoardId = BoardId([1; 8]);
/// Some other board.
pub const OTHER: BoardId = BoardId([2; 8]);

/// Random inbox runs feed at most this many datagrams.
pub const MAX_DATAGRAMS: usize = 32;

/// Generated stroke points and sync payloads have at most this many bytes, so every generated
/// packet fits in one datagram.
pub const MAX_PAYLOAD: usize = 64;

/// Random datagrams are up to this many bytes: past `MAX_DATAGRAM`, as a hostile sender may send.
pub const MAX_RANDOM: usize = 2 * wb_net::config::MAX_DATAGRAM;

/// Header and body framing bytes before a discovery payload of 128 to 16383 bytes.
pub const OVERHEAD: usize = wb_net::config::MAGIC.len() + 1 + 1 + 2;

/// Time between reject log lines for the same reason.
pub const EVERY: u64 = wb_net::config::REJECT_LOG_EVERY_MS;
