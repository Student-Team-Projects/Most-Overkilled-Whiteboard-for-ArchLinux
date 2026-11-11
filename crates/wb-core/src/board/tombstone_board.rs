//! The `TombstoneBoard` board.

use std::collections::{BTreeMap, BTreeSet};

use super::Board;
use super::state_hash::StateHash;
use crate::page::pages_to_hold;
use crate::{Lamport, Op, OpId, PeerId, Stamp, Stroke, StrokeId};

/// Keeps every stroke ever added, every erase and every add-page op, and hides the erased
/// strokes.
#[derive(Debug, Default)]
pub struct TombstoneBoard {
    strokes: BTreeMap<StrokeId, Stroke>,
    order: BTreeSet<(Lamport, StrokeId)>,
    erases: BTreeMap<OpId, StrokeId>,
    tombstones: BTreeSet<StrokeId>,
    page_adds: BTreeMap<OpId, u32>,
    /// The most pages any op asked for or any stroke needs; 0 until one does.
    pages: u32,
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
        let pages_grew = self.grow_to(pages_to_hold(stroke.bbox.max.y));
        let key = (stroke.lamport, id);
        self.strokes.insert(id, stroke);
        self.hash.insert_added(id);
        let shown = !self.tombstones.contains(&id) && self.order.insert(key);
        shown || pages_grew
    }

    fn tombstone(&mut self, id: OpId, target: StrokeId) -> bool {
        self.observe(id);
        // Taken before its Add arrives, or `stamp` could reuse it and be born erased.
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

    fn add_page(&mut self, id: OpId, count: u32) -> bool {
        self.observe(id);
        self.page_adds.insert(id, count);
        self.grow_to(count)
    }

    /// Makes the board at least `pages` long and returns whether that added pages.
    fn grow_to(&mut self, pages: u32) -> bool {
        let grew = pages > self.page_count();
        self.pages = self.pages.max(pages);
        grew
    }

    fn observe(&mut self, id: OpId) {
        let next = self.next_seq.entry(id.peer).or_default();
        // Saturates only on a foreign `seq` of `u64::MAX`.
        *next = (*next).max(id.seq.saturating_add(1));
    }
}

impl Board for TombstoneBoard {
    fn stamp(&mut self, me: PeerId) -> Stamp {
        let id = OpId::new(me, self.next_seq.get(&me).copied().unwrap_or_default());
        self.observe(id);
        // Saturates only after an unvalidated foreign op with Lamport `u64::MAX`.
        self.max_lamport = self.max_lamport.saturating_add(1);
        Stamp {
            id,
            lamport: Lamport(self.max_lamport),
        }
    }

    fn apply(&mut self, op: Op) -> bool {
        match op {
            Op::Add(stroke) => self.add(stroke),
            Op::Erase { id, target } => self.tombstone(id, target),
            Op::AddPage { id, count } => self.add_page(id, count),
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
        for (&id, &count) in &other.page_adds {
            self.add_page(id, count);
        }
    }

    fn visible(&self) -> impl Iterator<Item = &Stroke> {
        // Ties in `(lamport, peer)`, which only unvalidated foreign ops create, break by `seq`.
        self.order.iter().filter_map(|(_, id)| self.strokes.get(id))
    }

    fn page_count(&self) -> u32 {
        self.pages.max(1)
    }

    fn state_hash(&self) -> u128 {
        self.hash.value(self.page_count())
    }
}

// Reads the private op maps: kept erase and add-page ops are not part of the public API.
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::TombstoneBoard;
    use crate::{Aabb, Board, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};

    const TARGET: OpId = OpId::new(PeerId(0), 0);
    const ERASE: OpId = OpId::new(PeerId(1), 0);
    const ERASE_AGAIN: OpId = OpId::new(PeerId(2), 0);
    const PAGE_ADD: OpId = OpId::new(PeerId(3), 0);
    const SMALLER_PAGE_ADD: OpId = OpId::new(PeerId(4), 0);
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

    #[test]
    fn retains_add_page_ops_with_op_id_and_count() {
        let expected = BTreeMap::from([(PAGE_ADD, 3), (SMALLER_PAGE_ADD, 2)]);

        let mut asked = TombstoneBoard::new();
        for (&id, &count) in &expected {
            asked.apply(Op::AddPage { id, count });
        }
        assert_eq!(
            asked.page_adds, expected,
            "through apply, the smaller count too"
        );

        let mut merged = TombstoneBoard::new();
        merged.merge(&asked);
        assert_eq!(merged.page_adds, expected, "through merge");
    }
}
