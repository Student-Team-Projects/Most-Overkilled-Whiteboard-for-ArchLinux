//! `TombstoneBoard`: the layer-0 `Board`, a set of strokes plus a set of erased ids.

use std::collections::{BTreeMap, BTreeSet};

use super::state_hash::StateHash;
use super::{Board, ItemHasher, Xxh3};
use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, StrokeId};

/// A `Board` that keeps every stroke ever added and every erase op, and hides erased strokes.
///
/// `H` hashes each item of the state hash; peers of one board must use the same `H`. Callers
/// that do not care write plain `TombstoneBoard`, which uses XXH3-128. Ordered collections keep
/// iteration deterministic, so the simulator replays exactly.
#[derive(Debug)]
pub struct TombstoneBoard<H = Xxh3> {
    /// Every stroke ever added, erased or not.
    strokes: BTreeMap<StrokeId, Stroke>,
    /// Added and unerased strokes in draw order `(lamport, peer, seq)`.
    order: BTreeSet<(Lamport, StrokeId)>,
    /// Every erase op: its id → its target.
    erases: BTreeMap<OpId, StrokeId>,
    /// Erased stroke ids, whether or not their Add has arrived.
    tombstones: BTreeSet<StrokeId>,
    /// Next unused sequence number per peer seen.
    next_seq: BTreeMap<PeerId, u64>,
    /// Highest Lamport timestamp seen.
    max_lamport: u64,
    /// Hash of the canonical state.
    hash: StateHash<H>,
}

impl TombstoneBoard {
    /// An empty board hashed with XXH3-128. For another hasher `H`, use
    /// `TombstoneBoard::<H>::default()`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

// Not derived: a derive would require `H: Default`, and a hasher is never instantiated.
impl<H> Default for TombstoneBoard<H> {
    fn default() -> Self {
        Self {
            strokes: BTreeMap::new(),
            order: BTreeSet::new(),
            erases: BTreeMap::new(),
            tombstones: BTreeSet::new(),
            next_seq: BTreeMap::new(),
            max_lamport: 0,
            hash: StateHash::default(),
        }
    }
}

impl<H: ItemHasher> Board for TombstoneBoard<H> {
    /// Lamport saturates at `u64::MAX`, which only an unvalidated foreign op can reach.
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

    /// Ties in `(lamport, peer)`, which only unvalidated foreign ops create, break by `seq`.
    fn visible(&self) -> impl Iterator<Item = &Stroke> {
        self.order.iter().filter_map(|(_, id)| self.strokes.get(id))
    }

    fn state_hash(&self) -> u128 {
        self.hash.value()
    }

    /// In op id order.
    fn erase_ops(&self) -> impl Iterator<Item = (OpId, StrokeId)> {
        self.erases.iter().map(|(&id, &target)| (id, target))
    }
}

impl<H: ItemHasher> TombstoneBoard<H> {
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
        // The target's id is taken even before its Add arrives.
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

    /// Records `id` as used, so `next_op_id` never hands it out again.
    fn observe(&mut self, id: OpId) {
        let next = self.next_seq.entry(id.peer).or_default();
        // Saturates only on a foreign `seq` of `u64::MAX`, like the Lamport in `draw`.
        *next = (*next).max(id.seq.saturating_add(1));
    }

    fn next_op_id(&self, me: PeerId) -> OpId {
        OpId::new(me, self.next_seq.get(&me).copied().unwrap_or_default())
    }
}
