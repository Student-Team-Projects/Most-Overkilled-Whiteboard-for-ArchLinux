//! Test API for `Board`: op builders, generated op universes, and oracles that compute from the
//! ops alone what a board must show.

use super::Board;
use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};

/// Stroke fields no `Board` test looks at.
const COLOR: Rgba = Rgba([0, 0, 0, 255]);
const WIDTH: u32 = 16;
const BBOX: Aabb = Aabb {
    min: WorldPos { x: 0, y: 0 },
    max: WorldPos { x: 0, y: 0 },
};

/// Id of op number `seq` by `peer`.
pub fn id(peer: u64, seq: u64) -> OpId {
    OpId {
        peer: PeerId(peer),
        seq,
    }
}

/// Add of a stroke with no points.
pub fn add(id: OpId, lamport: u64) -> Op {
    Op::Add(Stroke {
        id,
        lamport: Lamport(lamport),
        color: COLOR,
        width: WIDTH,
        bbox: BBOX,
        points: EncodedPoints(Vec::new()),
    })
}

/// Erase op `id` of the stroke `target`.
pub fn erase(id: OpId, target: OpId) -> Op {
    Op::Erase { id, target }
}

/// The op's own id: a stroke's id for an Add.
pub fn op_id(op: &Op) -> OpId {
    match op {
        Op::Add(stroke) => stroke.id,
        Op::Erase { id, .. } => *id,
    }
}

impl Board {
    /// A board with `ops` applied in order.
    #[must_use]
    pub fn of(ops: &[Op]) -> Self {
        let mut board = Self::new();
        for op in ops {
            board.apply(op.clone());
        }
        board
    }

    /// A new board with `self`, then `other`, merged into it.
    #[must_use]
    pub fn merged(&self, other: &Self) -> Self {
        let mut board = Self::new();
        board.merge(self);
        board.merge(other);
        board
    }

    /// The visible strokes, owned, so pictures compare with `==`.
    #[must_use]
    pub fn picture(&self) -> Vec<Stroke> {
        self.visible().cloned().collect()
    }

    /// Draws a stroke with no points by `me` and returns it.
    pub fn draw_any(&mut self, me: PeerId) -> Stroke {
        let Op::Add(stroke) = self.draw(me, COLOR, WIDTH, EncodedPoints(Vec::new()), BBOX) else {
            unreachable!("draw returns an Add")
        };
        stroke
    }
}

/// Generated op universes: ids are unique and spread over 3 peers, Lamports collide on purpose,
/// and an erase targets any id: an Add, an Erase, a later op, or one that never exists.
pub mod strategy {
    use proptest::prelude::*;

    use super::{add, erase, id};
    use crate::{Op, OpId};

    /// At most this many ops in one universe.
    const UNIVERSE: usize = 40;
    /// Erase targets range over ids `0..48`: a full universe plus some that never exist.
    const TARGETS: u64 = 48;

    /// Universe op `i` has id `(i % 3, i)`.
    fn universe_id(i: u64) -> OpId {
        id(i % 3, i)
    }

    pub fn ops() -> impl Strategy<Value = Vec<Op>> {
        let spec = (any::<bool>(), 0..4u64, 0..TARGETS);
        prop::collection::vec(spec, 0..=UNIVERSE).prop_map(|specs| {
            (0u64..)
                .zip(specs)
                .map(|(i, (is_add, lamport, target))| {
                    if is_add {
                        add(universe_id(i), lamport)
                    } else {
                        erase(universe_id(i), universe_id(target))
                    }
                })
                .collect()
        })
    }

    /// A universe, and the same ops shuffled with each one delivered twice, as gossip may.
    pub fn permutations() -> impl Strategy<Value = (Vec<Op>, Vec<Op>)> {
        ops().prop_flat_map(|ops| {
            let delivered = [&ops[..], &ops[..]].concat();
            (Just(ops), Just(delivered).prop_shuffle())
        })
    }

    /// The ops of three replicas, each a random subset of one universe.
    pub fn replicas() -> impl Strategy<Value = [Vec<Op>; 3]> {
        let holders = prop::collection::vec(any::<[bool; 3]>(), UNIVERSE);
        (ops(), holders).prop_map(|(ops, holders)| {
            std::array::from_fn(|replica| {
                ops.iter()
                    .zip(&holders)
                    .filter(|(_, holds)| holds[replica])
                    .map(|(op, _)| op.clone())
                    .collect()
            })
        })
    }
}

/// What a board built from some ops must show, computed from the ops alone.
pub mod oracle {
    use std::collections::{BTreeMap, BTreeSet};

    use super::op_id;
    use crate::board::state_hash::StateHash;
    use crate::{Lamport, Op, OpId, Stroke};

    fn strokes(ops: &[Op]) -> impl Iterator<Item = &Stroke> {
        ops.iter().filter_map(|op| match op {
            Op::Add(stroke) => Some(stroke),
            Op::Erase { .. } => None,
        })
    }

    fn erase_targets(ops: &[Op]) -> BTreeSet<OpId> {
        erase_ops(ops).into_values().collect()
    }

    /// Unerased strokes sorted by `(lamport, peer, seq)`.
    pub fn picture(ops: &[Op]) -> Vec<Stroke> {
        let erased = erase_targets(ops);
        let mut picture: Vec<Stroke> = strokes(ops)
            .filter(|stroke| !erased.contains(&stroke.id))
            .cloned()
            .collect();
        picture.sort_by_key(|stroke| (stroke.lamport, stroke.id));
        picture
    }

    /// Hash of the added ids plus the erase targets, each counted once.
    pub fn state_hash(ops: &[Op]) -> u128 {
        let added: BTreeSet<OpId> = strokes(ops).map(|stroke| stroke.id).collect();
        let mut hash = StateHash::default();
        for id in added {
            hash.insert_added(id);
        }
        for target in erase_targets(ops) {
            hash.insert_tombstoned(target);
        }
        hash.value()
    }

    /// Every erase op: its id → its target.
    pub fn erase_ops(ops: &[Op]) -> BTreeMap<OpId, OpId> {
        ops.iter()
            .filter_map(|op| match op {
                Op::Erase { id, target } => Some((*id, *target)),
                Op::Add(_) => None,
            })
            .collect()
    }

    /// Every id the ops use: op ids and erase targets.
    pub fn ids(ops: &[Op]) -> BTreeSet<OpId> {
        ops.iter().map(op_id).chain(erase_targets(ops)).collect()
    }

    pub fn max_lamport(ops: &[Op]) -> Option<Lamport> {
        strokes(ops).map(|stroke| stroke.lamport).max()
    }
}
