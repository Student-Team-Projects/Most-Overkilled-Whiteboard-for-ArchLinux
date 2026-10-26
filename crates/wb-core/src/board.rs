//! The replicated board state: the `Board` trait and its impls.

mod hasher;
mod state_hash;
mod tombstone_board;

pub use hasher::{ItemHasher, Xxh3};
pub use tombstone_board::TombstoneBoard;

use crate::{Aabb, EncodedPoints, Op, OpId, PeerId, Rgba, Stroke, StrokeId};

/// One peer's replica of a shared board: every stroke ever added, and every erase.
///
/// Peers exchange the ops that `draw` and `erase` return and feed them to `apply` in any order,
/// any number of times; replicas that have seen the same ops show the same picture. A replica
/// never decodes stroke points, and no method fails or panics. `Default` is the empty board.
///
/// # Guarantees
///
/// 1. `merge` is commutative, associative and idempotent: merging the same replicas in any order
///    or grouping, any number of times, gives the same `visible` picture.
/// 2. `apply` is idempotent, and applying a set of ops in any order gives the same `visible`
///    picture.
/// 3. Erase wins: once an erase of a stroke is applied, that stroke is never visible, even when
///    its Add arrives later.
/// 4. `visible` yields strokes sorted by `(lamport, peer)`, so every replica stacks overlapping
///    strokes the same way.
/// 5. `draw` gives a Lamport timestamp greater than every one this replica has seen, so a new
///    stroke lies on top of everything its author could see. Op ids that `draw` and `erase` hand
///    out for one author never repeat, and never reuse an id this replica has seen as an op or as
///    an erase target.
/// 6. `state_hash` is a 128-bit hash of the canonical state: the ids of all strokes ever added
///    plus the ids of all erased targets. A new stroke id or a newly erased target updates it in
///    O(1); a duplicate Add or another erase of an erased target leaves it unchanged; erase op
///    ids are not hashed. Replicas with equal canonical state have equal hashes.
/// 7. Every erase op is kept with its op id and target, even when the target's Add has not
///    arrived, so anti-entropy can resend it (`erase_ops`).
///
/// # Not guaranteed
///
/// - Anything about the points: `EncodedPoints` stay opaque bytes.
/// - The order of independently drawn pieces of one split stroke: a concurrent stroke may be
///   stacked between them where they cross.
/// - Validation of foreign ops: there is no authentication, and a malformed op (a reused id, a
///   counter at `u64::MAX`) is stored as it comes.
/// - Memory ever shrinking: erased strokes and erase ops stay forever.
///
/// See also: `docs/contracts/board.md`, ADR 0001, 0008, 0009, 0026.
pub trait Board: Default {
    /// Draws a new stroke by `me` on top of everything this replica has seen, applies it, and
    /// returns its Add op for broadcast.
    ///
    /// The stroke gets the next unused op id of `me` and a Lamport timestamp above every one seen.
    /// The caller passes a `bbox` that covers `points`; the board never checks it.
    #[must_use = "the op must be broadcast"]
    fn draw(
        &mut self,
        me: PeerId,
        color: Rgba,
        width: u32,
        points: EncodedPoints,
        bbox: Aabb,
    ) -> Op;

    /// Erases `target` by `me`, applies the erase, and returns its op for broadcast.
    ///
    /// The erase gets the next unused op id of `me`. `target` need not have arrived yet.
    #[must_use = "the op must be broadcast"]
    fn erase(&mut self, me: PeerId, target: StrokeId) -> Op;

    /// Applies any op, local or remote, and returns whether the visible picture changed.
    ///
    /// A duplicate op changes nothing and returns `false`.
    fn apply(&mut self, op: Op) -> bool;

    /// Unions `other`, a replica of the same board, into this one: afterwards this replica holds
    /// every stroke and erase that either held.
    fn merge(&mut self, other: &Self);

    /// Strokes that were added and not erased, bottom first: sorted by `(lamport, peer)`.
    fn visible(&self) -> impl Iterator<Item = &Stroke>;

    /// The 128-bit hash of the canonical state, maintained as ops arrive (guarantee 6).
    #[must_use]
    fn state_hash(&self) -> u128;

    /// Every erase op held, as `(op id, target)`, each exactly once, in no promised order.
    fn erase_ops(&self) -> impl Iterator<Item = (OpId, StrokeId)>;
}
