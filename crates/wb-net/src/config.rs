//! Protocol constants: every peer must use the same values.

/// First 4 bytes of every datagram; a datagram starting with anything else is not ours.
pub const MAGIC: [u8; 4] = *b"WBRD";

/// The one wire version this build sends and accepts. Every wire change bumps it, the stroke
/// codec included; peers on different versions drop each other's packets.
pub const VERSION: u8 = 1;

/// Largest datagram any transport sends, in bytes: small enough to cross the internet without
/// IP fragmentation.
pub const MAX_DATAGRAM: usize = 1200;

/// A reason gets at most one log line about dropped datagrams per this many milliseconds.
pub const REJECT_LOG_EVERY_MS: u64 = 10_000;
