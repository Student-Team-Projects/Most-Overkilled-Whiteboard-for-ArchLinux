//! Generated op universes: ids are unique and spread over a few peers, Lamports collide on
//! purpose, and an erase targets any id: an Add, an Erase, a later op, or one that never exists.

use proptest::prelude::*;
use wb_core::{Op, OpId};

use super::{OpExt, OpIdExt};
use crate::support::config::{LAMPORTS, PEERS, TARGETS, UNIVERSE};

/// Universe op `i` has id `(i % PEERS, i)`.
fn universe_id(i: u64) -> OpId {
    OpId::of(i % PEERS, i)
}

/// A universe of ops, in id order.
pub fn ops() -> impl Strategy<Value = Vec<Op>> {
    let spec = (any::<bool>(), 0..LAMPORTS, 0..TARGETS);
    prop::collection::vec(spec, 0..=UNIVERSE).prop_map(|specs| {
        (0u64..)
            .zip(specs)
            .map(|(i, (is_add, lamport, target))| {
                if is_add {
                    Op::add(universe_id(i), lamport)
                } else {
                    Op::erase(universe_id(i), universe_id(target))
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
