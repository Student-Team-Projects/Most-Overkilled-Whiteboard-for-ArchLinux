//! Op ids `TombstoneBoard` hands out: numbered from zero, and saturated by foreign counters.

use wb_core::{Board, Op, OpId, TombstoneBoard};

use crate::support::board::{BoardExt, OpExt, OpIdExt};
use crate::support::config::ME;

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

#[test]
fn saturated_foreign_counters_do_not_panic() {
    let mut board = TombstoneBoard::of(&[Op::add(OpId::new(ME, u64::MAX), u64::MAX)]);
    let stroke = board.draw_any(ME);
    assert_eq!(stroke.lamport.0, u64::MAX);
    // Foreign ops are not validated, so the saturated seq hands out the foreign id again.
    assert_eq!(stroke.id, OpId::new(ME, u64::MAX));
    let _ = board.erase(ME, OpId::of(0, 0));
}
