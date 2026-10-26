//! The hash function behind `Board::state_hash`, swappable without touching the board.

use xxhash_rust::xxh3::xxh3_128;

/// A 128-bit hash of one item of a board's canonical state: an added stroke id or an erased
/// target.
///
/// `TombstoneBoard` sums these per-item hashes, so each new item updates its state hash in O(1).
/// Peers compare state hashes, so an impl must return the same value for the same bytes on every
/// peer, build and platform (std's hashers promise neither that nor 128 bits), and all peers of
/// a board must use the same impl.
pub trait ItemHasher {
    /// The hash of `bytes`.
    fn hash128(bytes: &[u8]) -> u128;
}

/// XXH3-128, the default `ItemHasher`: fast, and specified, so every peer computes the same value.
#[derive(Debug, Default, Clone, Copy)]
pub struct Xxh3;

impl ItemHasher for Xxh3 {
    fn hash128(bytes: &[u8]) -> u128 {
        xxh3_128(bytes)
    }
}
