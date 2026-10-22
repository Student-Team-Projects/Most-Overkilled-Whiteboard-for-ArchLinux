//! The replicated board state (`docs/contracts/board.md`, ADR 0001, 0008).

use std::collections::{BTreeMap, BTreeSet};

use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, StrokeId};

/// Domain tags so a stroke id and a tombstoned target with the same id hash differently.
const ADDED: u64 = 1;
const TOMBSTONED: u64 = 2;

/// One replica of a board: every stroke ever added plus every erase op (ADR 0001).
///
/// Ordered collections keep iteration deterministic, so the simulator replays exactly.
#[derive(Debug, Clone, Default)]
pub struct Board {
    /// Every stroke ever added, erased or not.
    strokes: BTreeMap<StrokeId, Stroke>,
    /// Added and unerased strokes in draw order `(lamport, peer, seq)` (ADR 0008).
    order: BTreeSet<(Lamport, StrokeId)>,
    /// Every erase op: its id → its target (guarantee 7).
    erases: BTreeMap<OpId, StrokeId>,
    /// Erased stroke ids, whether or not their Add has arrived (guarantee 3).
    tombstones: BTreeSet<StrokeId>,
    /// Next unused sequence number per peer seen (guarantee 5).
    next_seq: BTreeMap<PeerId, u64>,
    /// Highest Lamport timestamp seen (guarantee 5).
    max_lamport: u64,
    /// Wrapping sum of element hashes of the canonical state (guarantee 6).
    hash: u128,
}

impl Board {
    /// An empty board.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Draws a new stroke by `me` on top of everything seen, applies it and returns its Add op.
    ///
    /// The caller passes a `bbox` that covers `points`; the board never decodes them.
    /// Contract: `board.md` → `draw`, guarantee 5.
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
            // Saturates only on a foreign Lamport of `u64::MAX`; foreign ops are not validated.
            lamport: Lamport(self.max_lamport.saturating_add(1)),
            color,
            width,
            bbox,
            points,
        });
        self.apply(op.clone());
        op
    }

    /// Erases `target` by `me`, applies the erase and returns its op.
    ///
    /// Contract: `board.md` → `erase`, guarantees 3 and 5.
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
    /// Contract: `board.md` → `apply`, guarantees 2, 3, 6 and 7.
    pub fn apply(&mut self, op: Op) -> bool {
        match op {
            Op::Add(stroke) => self.add(stroke),
            Op::Erase { id, target } => self.tombstone(id, target),
        }
    }

    /// Unions `other` (a replica of the same board) into this one.
    ///
    /// Contract: `board.md` → `merge`, guarantee 1.
    pub fn merge(&mut self, other: &Board) {
        for (id, stroke) in &other.strokes {
            if !self.strokes.contains_key(id) {
                self.add(stroke.clone());
            }
        }
        for (&id, &target) in &other.erases {
            self.tombstone(id, target);
        }
    }

    /// Added and unerased strokes in draw order `(lamport, peer)`.
    ///
    /// Contract: `board.md` → `visible`, guarantees 3 and 4.
    pub fn visible(&self) -> impl Iterator<Item = &Stroke> {
        self.order.iter().filter_map(|(_, id)| self.strokes.get(id))
    }

    /// 128-bit hash of the canonical state: stroke ids ever added plus tombstoned targets.
    ///
    /// Contract: `board.md` → `state_hash`, guarantee 6; `anti-entropy.md` guarantee 5.
    #[must_use]
    pub fn state_hash(&self) -> u128 {
        self.hash
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
        self.hash = self.hash.wrapping_add(element_hash(ADDED, id));
        !self.tombstones.contains(&id) && self.order.insert(key)
    }

    fn tombstone(&mut self, id: OpId, target: StrokeId) -> bool {
        self.observe(id);
        self.erases.insert(id, target);
        if !self.tombstones.insert(target) {
            return false;
        }
        self.hash = self.hash.wrapping_add(element_hash(TOMBSTONED, target));
        self.strokes
            .get(&target)
            .is_some_and(|stroke| self.order.remove(&(stroke.lamport, target)))
    }

    /// Records `id` as used, so `next_op_id` never hands it out again.
    fn observe(&mut self, id: OpId) {
        let next = self.next_seq.entry(id.peer).or_default();
        *next = (*next).max(id.seq.saturating_add(1));
    }

    fn next_op_id(&self, me: PeerId) -> OpId {
        OpId {
            peer: me,
            seq: self.next_seq.get(&me).copied().unwrap_or_default(),
        }
    }
}

/// 128-bit hash of one canonical-state element: two `splitmix64` chains with different seeds.
///
/// Not cryptographic; a rare collision is healed by anti-entropy range checks (`anti-entropy.md`
/// guarantee 1). The sum of these over a set is order-independent and updates in O(1).
fn element_hash(tag: u64, id: OpId) -> u128 {
    let half = |seed: u64| {
        [tag, id.peer.0, id.seq]
            .into_iter()
            .fold(seed, |h, word| splitmix64(h ^ word))
    };
    (u128::from(half(0x243F_6A88_85A3_08D3)) << 64) | u128::from(half(0x1319_8A2E_0370_7344))
}

/// The `splitmix64` finalizer (Steele et al.): a fast bijective bit mixer on `u64`.
fn splitmix64(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests;
