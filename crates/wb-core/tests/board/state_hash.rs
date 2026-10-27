//! The state hash: what it covers, and its exact values.

use proptest::prelude::*;
use wb_core::{Board, Op, OpId};

use crate::support::board::{BoardExt, OpExt, OpIdExt, oracle, strategy};
use crate::support::config::ME;

proptest! {
    #[test]
    fn state_hash_tracks_canonical_state_and_matches_equal_boards(
        (ops, delivered) in strategy::permutations(),
    ) {
        let mut board = Board::of(&ops);
        let hash = board.state_hash();
        prop_assert_eq!(hash, Board::of(&oracle::canonical(&ops)).state_hash());
        prop_assert_eq!(hash, Board::of(&delivered).state_hash());

        // Duplicate adds, and a second erase of every target by a new op id, change nothing.
        let mut log = ops.clone();
        for op in &ops {
            match op {
                Op::Add(_) => {
                    board.apply(op.clone());
                }
                Op::Erase { target, .. } => log.push(board.erase(ME, *target)),
            }
        }
        prop_assert_eq!(board.state_hash(), hash);

        // A new stroke id, then a new tombstone, each change it.
        let fresh = board.draw_any(ME);
        log.push(Op::Add(fresh.clone()));
        let with_stroke = board.state_hash();
        prop_assert_ne!(with_stroke, hash);
        log.push(board.erase(ME, fresh.id));
        prop_assert_ne!(board.state_hash(), with_stroke);
        prop_assert_eq!(
            board.state_hash(),
            Board::of(&oracle::canonical(&log)).state_hash()
        );
    }
}

/// Peers compare these values, so byte order, item kinds and the XXH3 variant are pinned. The
/// expected values come from the reference xxHash (Python `xxhash.xxh3_128`) over
/// `[kind, peer as u64 LE, seq as u64 LE]`.
#[test]
fn state_hash_values_are_stable() {
    const ADDED: u128 = 0x3c7b_5609_41c1_e7fa_8416_1e77_2001_0f6a;
    const TOMBSTONED: u128 = 0xa8b3_6da9_793e_67eb_2db8_516a_bd89_775e;
    let ops = [
        Op::add(OpId::of(1, 2), 0),
        Op::erase(OpId::of(5, 5), OpId::of(1, 2)),
    ];
    assert_eq!(Board::new().state_hash(), 0);
    assert_eq!(Board::of(&ops[..1]).state_hash(), ADDED);
    assert_eq!(Board::of(&ops[1..]).state_hash(), TOMBSTONED);
    assert_eq!(Board::of(&ops).state_hash(), ADDED.wrapping_add(TOMBSTONED));
}

#[test]
fn state_hash_tells_ids_apart() {
    let hash = |peer, seq| Board::of(&[Op::add(OpId::of(peer, seq), 0)]).state_hash();
    let base = hash(1, 2);
    assert_ne!(base, hash(9, 2), "peer");
    assert_ne!(base, hash(1, 9), "seq");
    assert_ne!(base, hash(2, 1), "fields swapped");
}
