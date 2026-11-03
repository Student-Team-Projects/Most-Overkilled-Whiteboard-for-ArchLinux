//! The `Board` trait and its impls.

mod state_hash;
mod tombstone_board;

pub use tombstone_board::TombstoneBoard;

use crate::{Op, PeerId, Stamp, Stroke};

/// One peer's replica of a shared board.
///
/// Every op goes in through `apply`, local or remote. Ops can arrive in any order and any number
/// of times; replicas that saw the same ops show the same picture. The board never decodes
/// stroke points, and nothing fails or panics.
pub trait Board: Default {
    /// Reserves a fresh op id for `me` and a Lamport above every one this board has seen.
    ///
    /// Build the local op from it with `Op::add` or `Op::erase`, then `apply` it. A stamp is
    /// never handed out twice, even if its op is never applied. Its id is one this board has
    /// never seen, as an op or as an erase target; its Lamport puts a new stroke on top.
    fn stamp(&mut self, me: PeerId) -> Stamp;

    /// Applies any op, local or remote, and returns whether the visible picture changed.
    ///
    /// Idempotent and order-independent. An erase hides its target for good, even if the
    /// target's Add arrives later.
    fn apply(&mut self, op: Op) -> bool;

    /// Unions `other`, a replica of the same board, into this one.
    ///
    /// Commutative, associative and idempotent.
    fn merge(&mut self, other: &Self);

    /// Strokes added and not erased, bottom first: sorted by `(lamport, peer)`.
    fn visible(&self) -> impl Iterator<Item = &Stroke>;

    /// A 128-bit hash of every stroke id ever added plus every erased target.
    ///
    /// Independent of op order, updated in O(1) per new item, unchanged by duplicate adds and
    /// repeated erases; boards with equal state have equal hashes.
    #[must_use]
    fn state_hash(&self) -> u128;
}
