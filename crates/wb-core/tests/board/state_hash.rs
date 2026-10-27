//! Exact state hash values of `TombstoneBoard`.

use wb_core::{Board, Op, OpId, TombstoneBoard};

use crate::support::board::{BoardExt, OpExt, OpIdExt};

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
    assert_eq!(TombstoneBoard::new().state_hash(), 0);
    assert_eq!(TombstoneBoard::of(&ops[..1]).state_hash(), ADDED);
    assert_eq!(TombstoneBoard::of(&ops[1..]).state_hash(), TOMBSTONED);
    assert_eq!(
        TombstoneBoard::of(&ops).state_hash(),
        ADDED.wrapping_add(TOMBSTONED)
    );
}

#[test]
fn state_hash_tells_ids_apart() {
    let hash = |peer, seq| TombstoneBoard::of(&[Op::add(OpId::of(peer, seq), 0)]).state_hash();
    let base = hash(1, 2);
    assert_ne!(base, hash(9, 2), "peer");
    assert_ne!(base, hash(1, 9), "seq");
    assert_ne!(base, hash(2, 1), "fields swapped");
}
