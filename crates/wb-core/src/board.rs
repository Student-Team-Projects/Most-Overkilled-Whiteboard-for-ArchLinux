//! The replicated board state.

mod state_hash;

use std::collections::{BTreeMap, BTreeSet};

use state_hash::StateHash;

use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, StrokeId};

/// One peer's replica of a shared board: every stroke ever added, and every erase.
///
/// Ops can arrive in any order and any number of times; replicas that saw the same ops show the
/// same picture. The board never decodes stroke points, and nothing fails or panics.
#[derive(Debug, Default)]
pub struct Board {
    strokes: BTreeMap<StrokeId, Stroke>,
    order: BTreeSet<(Lamport, StrokeId)>,
    erases: BTreeMap<OpId, StrokeId>,
    tombstones: BTreeSet<StrokeId>,
    next_seq: BTreeMap<PeerId, u64>,
    max_lamport: u64,
    hash: StateHash,
}

impl Board {
    /// An empty board.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Draws a stroke by `me`, applies it and returns its Add op for broadcast.
    ///
    /// Its Lamport is above every one this board has seen, so it lies on top, and its op id is
    /// one this board has never seen, as an op or as an erase target. `bbox` must cover `points`;
    /// it is not checked.
    #[must_use = "the op must be broadcast"]
    pub fn draw(
        &mut self,
        me: PeerId,
        color: Rgba,
        width: u32,
        points: EncodedPoints,
        bbox: Aabb,
    ) -> Op {
        let op = Op::Add(Stroke {
            id: self.next_op_id(me),
            // Saturates only after an unvalidated foreign op with Lamport `u64::MAX`.
            lamport: Lamport(self.max_lamport.saturating_add(1)),
            color,
            width,
            bbox,
            points,
        });
        self.apply(op.clone());
        op
    }

    /// Erases `target` by `me`, applies the erase and returns its op for broadcast.
    ///
    /// The erase gets an op id this board has never seen. `target` need not have arrived yet.
    #[must_use = "the op must be broadcast"]
    pub fn erase(&mut self, me: PeerId, target: StrokeId) -> Op {
        let op = Op::Erase {
            id: self.next_op_id(me),
            target,
        };
        self.apply(op.clone());
        op
    }

    /// Applies any op, local or remote, and returns whether the visible picture changed.
    ///
    /// Idempotent and order-independent. An erase hides its target for good, even if the
    /// target's Add arrives later.
    pub fn apply(&mut self, op: Op) -> bool {
        match op {
            Op::Add(stroke) => self.add(stroke),
            Op::Erase { id, target } => self.tombstone(id, target),
        }
    }

    /// Unions `other`, a replica of the same board, into this one.
    ///
    /// Commutative, associative and idempotent.
    pub fn merge(&mut self, other: &Self) {
        for (id, stroke) in &other.strokes {
            if !self.strokes.contains_key(id) {
                self.add(stroke.clone());
            }
        }
        for (&id, &target) in &other.erases {
            self.tombstone(id, target);
        }
    }

    /// Strokes added and not erased, bottom first: sorted by `(lamport, peer)`.
    pub fn visible(&self) -> impl Iterator<Item = &Stroke> {
        // Ties in `(lamport, peer)`, which only unvalidated foreign ops create, break by `seq`.
        self.order.iter().filter_map(|(_, id)| self.strokes.get(id))
    }

    /// A 128-bit hash of every stroke id ever added plus every erased target.
    ///
    /// Independent of op order, updated in O(1) per new item, unchanged by duplicate adds and
    /// repeated erases; boards with equal state have equal hashes.
    #[must_use]
    pub fn state_hash(&self) -> u128 {
        self.hash.value()
    }

    fn add(&mut self, stroke: Stroke) -> bool {
        let id = stroke.id;
        self.observe(id);
        self.max_lamport = self.max_lamport.max(stroke.lamport.0);
        if self.strokes.contains_key(&id) {
            return false;
        }
        let key = (stroke.lamport, id);
        self.strokes.insert(id, stroke);
        self.hash.insert_added(id);
        !self.tombstones.contains(&id) && self.order.insert(key)
    }

    fn tombstone(&mut self, id: OpId, target: StrokeId) -> bool {
        self.observe(id);
        // Taken before its Add arrives, or `draw` could reuse it and be born erased.
        self.observe(target);
        self.erases.insert(id, target);
        if !self.tombstones.insert(target) {
            return false;
        }
        self.hash.insert_tombstoned(target);
        self.strokes
            .get(&target)
            .is_some_and(|stroke| self.order.remove(&(stroke.lamport, target)))
    }

    fn observe(&mut self, id: OpId) {
        let next = self.next_seq.entry(id.peer).or_default();
        // Saturates only on a foreign `seq` of `u64::MAX`.
        *next = (*next).max(id.seq.saturating_add(1));
    }

    fn next_op_id(&self, me: PeerId) -> OpId {
        OpId::new(me, self.next_seq.get(&me).copied().unwrap_or_default())
    }
}

// Reads the private erase map: kept erase ops are not part of the public API.
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Board;
    use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};

    const TARGET: OpId = OpId::new(PeerId(0), 0);
    const ERASE: OpId = OpId::new(PeerId(1), 0);
    const ERASE_AGAIN: OpId = OpId::new(PeerId(2), 0);
    const ORIGIN: WorldPos = WorldPos::new(0, 0);

    #[test]
    fn retains_erase_ops_with_op_id_and_target() {
        let expected = BTreeMap::from([(ERASE, TARGET), (ERASE_AGAIN, TARGET)]);

        let mut erased = Board::new();
        for (&id, &target) in &expected {
            erased.apply(Op::Erase { id, target });
        }
        assert_eq!(erased.erases, expected, "erase before add, through apply");

        let mut merged = Board::new();
        merged.merge(&erased);
        assert_eq!(merged.erases, expected, "erase before add, through merge");

        merged.apply(Op::Add(Stroke {
            id: TARGET,
            lamport: Lamport(1),
            color: Rgba([0, 0, 0, 255]),
            width: 1,
            bbox: Aabb::new(ORIGIN, ORIGIN),
            points: EncodedPoints(Vec::new()),
        }));
        assert_eq!(merged.erases, expected, "after the add");
    }
}
