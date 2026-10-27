//! Hash of a board's canonical state.

use xxhash_rust::xxh3::xxh3_128;

use crate::StrokeId;

/// 128-bit hash of a set of added stroke ids plus a set of erased target ids.
///
/// It is the wrapping sum of one hash per item, so the order items arrive in does not matter
/// and each insert is O(1). Callers insert each item once.
#[derive(Debug, Default)]
pub struct StateHash(u128);

/// Kind of item, hashed with its id so an added id and an erased target differ. The
/// discriminant is the hashed byte, so changing it changes every peer's hash.
#[derive(Clone, Copy)]
#[repr(u8)]
enum Item {
    Added = 1,
    Tombstoned = 2,
}

impl StateHash {
    /// Counts `id` as an added stroke id.
    pub fn insert_added(&mut self, id: StrokeId) {
        self.insert(Item::Added, id);
    }

    /// Counts `target` as an erased target.
    pub fn insert_tombstoned(&mut self, target: StrokeId) {
        self.insert(Item::Tombstoned, target);
    }

    /// The current hash.
    pub fn value(&self) -> u128 {
        self.0
    }

    /// Adds the hash of the kind byte, then the id's peer and seq as little-endian `u64`s.
    fn insert(&mut self, item: Item, id: StrokeId) {
        let mut bytes = [0; 17];
        bytes[0] = item as u8;
        bytes[1..9].copy_from_slice(&id.peer.0.to_le_bytes());
        bytes[9..].copy_from_slice(&id.seq.to_le_bytes());
        self.0 = self.0.wrapping_add(item_hash(&bytes));
    }
}

/// Peers compare state hashes, so this algorithm is part of the protocol: it must give the same
/// value on every peer, build and platform, and changing it needs a protocol version bump.
fn item_hash(bytes: &[u8]) -> u128 {
    xxh3_128(bytes)
}
