//! Hash of a board's canonical state (`board.md` guarantee 6, `anti-entropy.md` guarantee 5).

#[cfg(test)]
#[path = "state_hash_test.rs"]
mod test;

use xxhash_rust::xxh3::Xxh3;

use crate::StrokeId;

/// 128-bit hash of a set of added stroke ids plus a set of tombstoned target ids.
///
/// It is the wrapping sum of one hash per element, so the order elements arrive in does not
/// matter and each insert is O(1). Callers insert each element once.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) struct StateHash(u128);

/// Kind of element, hashed with its id so an added id and a tombstoned target differ. The
/// discriminant is the hashed byte, so changing it changes every peer's hash.
#[derive(Clone, Copy)]
#[repr(u8)]
enum Element {
    Added = 1,
    Tombstoned = 2,
}

impl StateHash {
    pub(super) fn insert_added(&mut self, id: StrokeId) {
        self.insert(Element::Added, id);
    }

    pub(super) fn insert_tombstoned(&mut self, target: StrokeId) {
        self.insert(Element::Tombstoned, target);
    }

    pub(super) fn value(self) -> u128 {
        self.0
    }

    fn insert(&mut self, element: Element, id: StrokeId) {
        self.0 = self.0.wrapping_add(element_hash(element, id));
    }
}

/// xxh3-128 of the element's kind byte, then the id's peer and seq as little-endian `u64`s.
///
/// xxh3 is a specified algorithm, so every peer and build computes the same value; std's hashers
/// promise neither that nor 128 bits. A rare collision is healed by anti-entropy range checks
/// (`anti-entropy.md` guarantee 1).
fn element_hash(element: Element, id: StrokeId) -> u128 {
    let mut hasher = Xxh3::new();
    hasher.update(&[element as u8]);
    hasher.update(&id.peer.0.to_le_bytes());
    hasher.update(&id.seq.to_le_bytes());
    hasher.digest128()
}
