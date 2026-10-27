//! The `Board` trait and its impls.

mod state_hash;
mod tombstone_board;

pub use tombstone_board::TombstoneBoard;

use crate::{Aabb, EncodedPoints, Op, PeerId, Rgba, Stroke, StrokeId};

/// One peer's replica of a shared board.
///
/// Ops can arrive in any order and any number of times; replicas that saw the same ops show the
/// same picture. The board never decodes stroke points, and nothing fails or panics.
pub trait Board: Default {
    /// Draws a stroke by `me`, applies it and returns its Add op for broadcast.
    ///
    /// Its Lamport is above every one this board has seen, so it lies on top, and its op id is
    /// one this board has never seen, as an op or as an erase target. `bbox` must cover `points`;
    /// it is not checked.
    #[must_use = "the op must be broadcast"]
    fn draw(
        &mut self,
        me: PeerId,
        color: Rgba,
        width: u32,
        points: EncodedPoints,
        bbox: Aabb,
    ) -> Op;

    /// Erases `target` by `me`, applies the erase and returns its op for broadcast.
    ///
    /// The erase gets an op id this board has never seen. `target` need not have arrived yet.
    #[must_use = "the op must be broadcast"]
    fn erase(&mut self, me: PeerId, target: StrokeId) -> Op;

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
