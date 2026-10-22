//! Test API for `Board`: op builders, generated op universes, and oracles that compute from the ops
//! alone what a board must show.

use super::Board;
use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};

const ORIGIN: WorldPos = WorldPos { x: 0, y: 0 };
const BBOX: Aabb = Aabb {
    min: ORIGIN,
    max: ORIGIN,
};

/// Id of op number `seq` by `peer`.
pub(crate) fn id(peer: u64, seq: u64) -> OpId {
    OpId {
        peer: PeerId(peer),
        seq,
    }
}

/// Add of a minimal stroke.
pub(crate) fn add(id: OpId, lamport: u64) -> Op {
    Op::Add(Stroke {
        id,
        lamport: Lamport(lamport),
        color: Rgba([0, 0, 0, 255]),
        width: 16,
        bbox: BBOX,
        points: EncodedPoints(Vec::new()),
    })
}

pub(crate) fn erase(id: OpId, target: OpId) -> Op {
    Op::Erase { id, target }
}

/// The op's own id: a stroke's id for an Add.
pub(crate) fn op_id(op: &Op) -> OpId {
    match op {
        Op::Add(stroke) => stroke.id,
        Op::Erase { id, .. } => *id,
    }
}

impl Board {
    /// A board with `ops` applied in order.
    pub(crate) fn of<'a>(ops: impl IntoIterator<Item = &'a Op>) -> Self {
        let mut board = Self::new();
        for op in ops {
            board.apply(op.clone());
        }
        board
    }

    /// A new board: an empty one with `self`, then `other`, merged in.
    pub(crate) fn merged(&self, other: &Self) -> Self {
        let mut board = Self::new();
        board.merge(self);
        board.merge(other);
        board
    }

    /// The visible strokes, owned, so pictures compare with `==`.
    pub(crate) fn picture(&self) -> Vec<Stroke> {
        self.visible().cloned().collect()
    }

    /// Draws a minimal stroke by `me` and returns it.
    pub(crate) fn draw_any(&mut self, me: PeerId) -> Stroke {
        let Op::Add(stroke) =
            self.draw(me, Rgba([255, 0, 0, 255]), 32, EncodedPoints(vec![1]), BBOX)
        else {
            unreachable!("draw returns an Add")
        };
        stroke
    }
}

/// Generated op universes: ids are unique and spread over 3 peers, Lamports collide on purpose, and
/// an erase targets any op id of the universe (an Add, an Erase, a later op) or one that never
/// exists.
pub(crate) mod strategy {
    use proptest::prelude::*;

    use super::{add, erase, id};
    use crate::{Op, OpId};

    /// Most ops one universe holds.
    const UNIVERSE: usize = 40;

    /// Universe op `i` has id `(i % 3, i)`.
    fn universe_id(i: u64) -> OpId {
        id(i % 3, i)
    }

    pub(crate) fn ops() -> impl Strategy<Value = Vec<Op>> {
        let spec = (any::<bool>(), 0..4u64, 0..48u64);
        prop::collection::vec(spec, 0..UNIVERSE).prop_map(|specs| {
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

    /// A universe and the same ops shuffled.
    pub(crate) fn permutations() -> impl Strategy<Value = (Vec<Op>, Vec<Op>)> {
        ops().prop_flat_map(|ops| (Just(ops.clone()), Just(ops).prop_shuffle()))
    }

    /// The ops of three replicas, each a random subset of one universe.
    pub(crate) fn replicas() -> impl Strategy<Value = [Vec<Op>; 3]> {
        (ops(), prop::collection::vec(any::<u8>(), UNIVERSE)).prop_map(|(ops, masks)| {
            std::array::from_fn(|replica| {
                ops.iter()
                    .zip(&masks)
                    .filter(|(_, mask)| *mask & (1 << replica) != 0)
                    .map(|(op, _)| op.clone())
                    .collect()
            })
        })
    }
}

/// What a board built from some ops must show, computed from the ops alone.
pub(crate) mod oracle {
    use std::collections::{BTreeMap, BTreeSet};

    use crate::board::state_hash::StateHash;
    use crate::{Lamport, Op, OpId, Stroke};

    fn erase_targets<'a>(ops: impl IntoIterator<Item = &'a Op>) -> BTreeSet<OpId> {
        erase_ops(ops).into_values().collect()
    }

    fn strokes<'a>(ops: impl IntoIterator<Item = &'a Op>) -> impl Iterator<Item = &'a Stroke> {
        ops.into_iter().filter_map(|op| match op {
            Op::Add(stroke) => Some(stroke),
            Op::Erase { .. } => None,
        })
    }

    /// Unerased strokes sorted by `(lamport, peer, seq)`.
    pub(crate) fn picture(ops: &[Op]) -> Vec<Stroke> {
        let erased = erase_targets(ops);
        let mut picture: Vec<Stroke> = strokes(ops)
            .filter(|stroke| !erased.contains(&stroke.id))
            .cloned()
            .collect();
        picture.sort_by_key(|stroke| (stroke.lamport, stroke.id));
        picture
    }

    /// Hash of the added ids plus the erase targets, each counted once.
    pub(crate) fn state_hash<'a>(ops: impl IntoIterator<Item = &'a Op> + Clone) -> u128 {
        let added: BTreeSet<OpId> = strokes(ops.clone()).map(|stroke| stroke.id).collect();
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
    pub(crate) fn erase_ops<'a>(ops: impl IntoIterator<Item = &'a Op>) -> BTreeMap<OpId, OpId> {
        ops.into_iter()
            .filter_map(|op| match op {
                Op::Erase { id, target } => Some((*id, *target)),
                Op::Add(_) => None,
            })
            .collect()
    }

    /// Every id the ops use: op ids and erase targets.
    pub(crate) fn ids<'a>(ops: impl IntoIterator<Item = &'a Op>) -> BTreeSet<OpId> {
        ops.into_iter()
            .flat_map(|op| match op {
                Op::Add(stroke) => [Some(stroke.id), None],
                Op::Erase { id, target } => [Some(*id), Some(*target)],
            })
            .flatten()
            .collect()
    }

    pub(crate) fn max_lamport<'a>(ops: impl IntoIterator<Item = &'a Op>) -> Option<Lamport> {
        strokes(ops).map(|stroke| stroke.lamport).max()
    }
}
