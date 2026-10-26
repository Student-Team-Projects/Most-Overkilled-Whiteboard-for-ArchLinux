//! `TombstoneBoard` beyond the `Board` suite: op numbering, foreign counters, and the state hash.

use proptest::prelude::*;
use wb_core::{Board, Op, OpId, TombstoneBoard};

use crate::support::board::{BoardExt, CountingHasher, OpExt, OpIdExt, oracle, strategy};
use crate::support::config::ME;

/// `TombstoneBoard` with its default hasher: a type alias applies the default type parameter,
/// a path in an expression (`TombstoneBoard::of`) does not.
type Plain = TombstoneBoard;

/// ADR 0009: one author numbers all its ops, Adds and Erases, 0, 1, 2…
#[test]
fn numbers_own_ops_from_zero() {
    let mut board = TombstoneBoard::new();
    let first = board.draw_any(ME);
    let erased = board.erase(ME, first.id);
    let second = board.draw_any(ME);
    assert_eq!(
        [first.id, erased.id(), second.id],
        [OpId::new(ME, 0), OpId::new(ME, 1), OpId::new(ME, 2)]
    );
}

/// Foreign ops are not validated: counters already at `u64::MAX` saturate instead of panicking.
#[test]
fn saturated_foreign_counters_do_not_panic() {
    let mut board = Plain::of(&[Op::add(OpId::new(ME, u64::MAX), u64::MAX)]);
    let stroke = board.draw_any(ME);
    assert_eq!(stroke.lamport.0, u64::MAX);
    // The saturated seq hands out the foreign id again (not guaranteed: unvalidated input).
    assert_eq!(stroke.id, OpId::new(ME, u64::MAX));
    let _ = board.erase(ME, OpId::of(0, 0));
}

proptest! {
    /// The state hash sums one item hash per canonical item: with every item hashing to 1, it
    /// counts them, however often and in whatever order ops arrive.
    #[test]
    fn state_hash_counts_each_item_once((ops, delivered) in strategy::permutations()) {
        let board = TombstoneBoard::<CountingHasher>::of(&delivered);
        prop_assert_eq!(board.state_hash(), oracle::canonical(&ops).len() as u128);
    }
}

/// Peers compare these values, so the hashed bytes must never change silently: byte order, item
/// kinds and the XXH3 variant are all pinned here. The expected values come from the reference
/// xxHash (Python `xxhash.xxh3_128`) over `[kind, peer as u64 LE, seq as u64 LE]`.
#[test]
fn state_hash_values_are_stable() {
    const ADDED: u128 = 0x3c7b_5609_41c1_e7fa_8416_1e77_2001_0f6a;
    const TOMBSTONED: u128 = 0xa8b3_6da9_793e_67eb_2db8_516a_bd89_775e;
    let ops = [
        Op::add(OpId::of(1, 2), 0),
        Op::erase(OpId::of(5, 5), OpId::of(1, 2)),
    ];
    assert_eq!(TombstoneBoard::new().state_hash(), 0);
    assert_eq!(Plain::of(&ops[..1]).state_hash(), ADDED);
    assert_eq!(Plain::of(&ops[1..]).state_hash(), TOMBSTONED);
    assert_eq!(Plain::of(&ops).state_hash(), ADDED.wrapping_add(TOMBSTONED));
}

/// Ids differing in one field, or with fields swapped, hash apart.
#[test]
fn state_hash_tells_ids_apart() {
    let hash = |peer, seq| Plain::of(&[Op::add(OpId::of(peer, seq), 0)]).state_hash();
    let base = hash(1, 2);
    assert_ne!(base, hash(9, 2), "peer");
    assert_ne!(base, hash(1, 9), "seq");
    assert_ne!(base, hash(2, 1), "fields swapped");
}
