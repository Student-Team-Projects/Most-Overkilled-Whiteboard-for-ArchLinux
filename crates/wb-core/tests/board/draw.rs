//! Local draws and erases: Lamport on top, and op ids that never repeat.

use std::collections::BTreeSet;

use proptest::prelude::*;
use wb_core::{Board, Op, OpId};

use crate::support::board::{BoardExt, OpExt, OpIdExt, oracle, strategy};
use crate::support::config::ME;

proptest! {
    #[test]
    fn draw_after_merge_is_on_top([a, b, _] in strategy::replicas()) {
        let seen = [&a[..], &b[..]].concat();
        let mut board = Board::of(&a).merged(&Board::of(&b));
        let first = board.draw_any(ME);
        prop_assert!(Some(first.lamport) > oracle::max_lamport(&seen));
        prop_assert_eq!(board.visible().last(), Some(&first));

        let erased = board.erase(ME, first.id);
        let second = board.draw_any(ME);
        prop_assert!(second.lamport > first.lamport);

        let mine = [first.id, erased.id(), second.id];
        let seen_ids = oracle::ids(&seen);
        prop_assert_eq!(BTreeSet::from(mine).len(), mine.len());
        prop_assert!(mine.iter().all(|id| !seen_ids.contains(id)));
    }
}

/// An erase target whose Add has not arrived is an id `draw` must not reuse, or the new stroke
/// would be born erased.
#[test]
fn draw_skips_ids_of_erase_targets() {
    let target = OpId::new(ME, 0);
    let mut board = Board::of(&[Op::erase(OpId::of(0, 0), target)]);
    let stroke = board.draw_any(ME);
    assert_ne!(stroke.id, target);
    assert_eq!(board.visible().last(), Some(&stroke));
}

#[test]
fn numbers_own_ops_from_zero() {
    let mut board = Board::new();
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
    let mut board = Board::of(&[Op::add(OpId::new(ME, u64::MAX), u64::MAX)]);
    let stroke = board.draw_any(ME);
    assert_eq!(stroke.lamport.0, u64::MAX);
    // Foreign ops are not validated, so the saturated seq hands out the foreign id again.
    assert_eq!(stroke.id, OpId::new(ME, u64::MAX));
    let _ = board.erase(ME, OpId::of(0, 0));
}
