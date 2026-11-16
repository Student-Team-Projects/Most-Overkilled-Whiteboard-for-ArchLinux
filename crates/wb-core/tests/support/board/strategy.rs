//! Generated op universes: ids are unique and spread over a few peers, Lamports collide on
//! purpose, an erase targets any id: an Add, an Erase, a later op, or one that never exists,
//! and strokes and add-page ops reach a few pages, gaps between them included.

use proptest::prelude::*;
use wb_core::{Op, OpId};

use super::{OpExt, OpIdExt};
use crate::support::config::{LAMPORTS, PAGE_PITCH, PAGES, TARGETS, UNIVERSE};

/// A universe of ops, in id order.
pub fn ops() -> impl Strategy<Value = Vec<Op>> {
    let spec = (0..3, 0..LAMPORTS, 0..TARGETS, 0..PAGES, 0..PAGE_PITCH);
    prop::collection::vec(spec, 0..=UNIVERSE).prop_map(|specs| {
        (0u64..)
            .zip(specs)
            .map(|(i, (kind, lamport, target, page, below_top))| {
                let id = OpId::universe(i);
                match kind {
                    0 => Op::stroke_down_to(id, lamport, i64::from(page) * PAGE_PITCH + below_top),
                    1 => Op::erasure(id, OpId::universe(target)),
                    _ => Op::page_add(id, page),
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
