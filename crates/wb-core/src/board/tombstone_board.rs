//! The `TombstoneBoard` board.

use std::collections::{BTreeMap, BTreeSet};

use super::Board;
use super::state_hash::StateHash;
use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, StrokeId};

/// Keeps every stroke ever added and every erase, and hides the erased strokes.
#[derive(Debug, Default)]
pub struct TombstoneBoard {
    strokes: BTreeMap<StrokeId, Stroke>,
    order: BTreeSet<(Lamport, StrokeId)>,
    erases: BTreeMap<OpId, StrokeId>,
    tombstones: BTreeSet<StrokeId>,
    next_seq: BTreeMap<PeerId, u64>,
    max_lamport: u64,
    hash: StateHash,
}

impl TombstoneBoard {
    /// An empty board.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

impl Board for TombstoneBoard {
    fn draw(
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

    fn erase(&mut self, me: PeerId, target: StrokeId) -> Op {
        let op = Op::Erase {
            id: self.next_op_id(me),
            target,
        };
        self.apply(op.clone());
        op
    }

    fn apply(&mut self, op: Op) -> bool {
        match op {
            Op::Add(stroke) => self.add(stroke),
            Op::Erase { id, target } => self.tombstone(id, target),
        }
    }

    fn merge(&mut self, other: &Self) {
        for (id, stroke) in &other.strokes {
            if !self.strokes.contains_key(id) {
                self.add(stroke.clone());
            }
        }
        for (&id, &target) in &other.erases {
            self.tombstone(id, target);
        }
    }

    fn visible(&self) -> impl Iterator<Item = &Stroke> {
        // Ties in `(lamport, peer)`, which only unvalidated foreign ops create, break by `seq`.
        self.order.iter().filter_map(|(_, id)| self.strokes.get(id))
    }

    fn state_hash(&self) -> u128 {
        self.hash.value()
    }
}

// Reads the private erase map: kept erase ops are not part of the public API.
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::TombstoneBoard;
    use crate::{Aabb, Board, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};

    const TARGET: OpId = OpId::new(PeerId(0), 0);
    const ERASE: OpId = OpId::new(PeerId(1), 0);
    const ERASE_AGAIN: OpId = OpId::new(PeerId(2), 0);
    const ORIGIN: WorldPos = WorldPos::new(0, 0);
    const LAMPORT: Lamport = Lamport(1);
    const COLOR: Rgba = Rgba([0, 0, 0, 255]);
    const WIDTH: u32 = 1;

    #[test]
    fn retains_erase_ops_with_op_id_and_target() {
        let expected = BTreeMap::from([(ERASE, TARGET), (ERASE_AGAIN, TARGET)]);

        let mut erased = TombstoneBoard::new();
        for (&id, &target) in &expected {
            erased.apply(Op::Erase { id, target });
        }
        assert_eq!(erased.erases, expected, "erase before add, through apply");

        let mut merged = TombstoneBoard::new();
        merged.merge(&erased);
        assert_eq!(merged.erases, expected, "erase before add, through merge");

        merged.apply(Op::Add(Stroke {
            id: TARGET,
            lamport: LAMPORT,
            color: COLOR,
            width: WIDTH,
            bbox: Aabb::new(ORIGIN, ORIGIN),
            points: EncodedPoints(Vec::new()),
        }));
        assert_eq!(merged.erases, expected, "after the add");
    }
}
