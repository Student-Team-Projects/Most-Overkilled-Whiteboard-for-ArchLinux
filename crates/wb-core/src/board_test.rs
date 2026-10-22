//! Conformance tests for `docs/contracts/board.md`; each test names the guarantee it checks.

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use super::Board;
use super::testing::{add, erase, id, op_id, oracle, strategy};
use crate::{Op, PeerId};

proptest! {
    /// Guarantee 1, plus a merged board's `state_hash` (guarantee 6).
    #[test]
    fn merge_laws(ops in strategy::replicas()) {
        let [a, b, c] = ops.each_ref().map(Board::of);
        let ab = a.merged(&b);
        prop_assert_eq!(ab.picture(), b.merged(&a).picture());
        prop_assert_eq!(ab.merged(&c).picture(), a.merged(&b.merged(&c)).picture());
        prop_assert_eq!(a.merged(&a).picture(), a.picture());

        prop_assert_eq!(ab.state_hash(), b.merged(&a).state_hash());
        prop_assert_eq!(ab.state_hash(), oracle::state_hash(ops[0].iter().chain(&ops[1])));
    }

    /// Guarantee 2; `apply` reports exactly when the picture changes.
    #[test]
    fn apply_any_permutation((ops, shuffled) in strategy::permutations()) {
        let mut board = Board::new();
        for op in &shuffled {
            let before = board.picture();
            let changed = board.apply(op.clone());
            prop_assert_eq!(changed, board.picture() != before);
        }
        let settled = board.picture();
        prop_assert_eq!(&settled, &Board::of(&ops).picture());
        for op in ops {
            prop_assert!(!board.apply(op));
        }
        prop_assert_eq!(board.picture(), settled);
    }

    /// Guarantees 3 and 4: `visible` is exactly the unerased adds, sorted by `(lamport, peer)`.
    #[test]
    fn visible_is_sorted(ops in strategy::ops()) {
        let board = Board::of(&ops);
        prop_assert!(board.visible().map(|s| (s.lamport, s.id.peer)).is_sorted());
        prop_assert_eq!(board.picture(), oracle::picture(&ops));
    }

    /// Guarantee 5: above every Lamport seen, and no id seen as an op or as an erase target.
    #[test]
    fn draw_after_merge_is_on_top([a, b, _] in strategy::replicas()) {
        let seen = || a.iter().chain(&b);
        let mut board = Board::of(&a).merged(&Board::of(&b));
        let me = PeerId(1);

        let first = board.draw_any(me);
        prop_assert!(Some(first.lamport) > oracle::max_lamport(seen()));
        prop_assert_eq!(board.visible().last(), Some(&first));

        let erased = board.erase(me, first.id);
        let second = board.draw_any(me);
        prop_assert!(second.lamport > first.lamport);

        let mine = [first.id, op_id(&erased), second.id];
        let seen_ids = oracle::ids(seen());
        prop_assert_eq!(BTreeSet::from(mine).len(), mine.len());
        prop_assert!(mine.iter().all(|id| !seen_ids.contains(id)));
    }

    /// Guarantee 6.
    #[test]
    fn state_hash_tracks_canonical_state_and_matches_equal_boards(
        (ops, shuffled) in strategy::permutations(),
    ) {
        let mut board = Board::of(&ops);
        let hash = board.state_hash();
        prop_assert_eq!(hash, oracle::state_hash(&ops));
        prop_assert_eq!(hash, Board::of(&shuffled).state_hash());

        // Duplicate adds, and a second erase of every target by a new op id, change nothing.
        let mut log = ops.clone();
        for op in &ops {
            match op {
                Op::Add(_) => prop_assert!(!board.apply(op.clone())),
                Op::Erase { target, .. } => log.push(board.erase(PeerId(7), *target)),
            }
        }
        prop_assert_eq!(board.state_hash(), hash);

        // A new stroke id, then a new tombstone, each change it.
        let fresh = board.draw_any(PeerId(7));
        let with_stroke = board.state_hash();
        prop_assert_ne!(with_stroke, hash);
        log.push(board.erase(PeerId(7), fresh.id));
        log.push(Op::Add(fresh));
        prop_assert_ne!(board.state_hash(), with_stroke);
        prop_assert_eq!(board.state_hash(), oracle::state_hash(&log));
    }

    /// Guarantee 7.
    #[test]
    fn retains_erase_ops_with_op_id_and_target(ops in strategy::ops()) {
        prop_assert_eq!(Board::of(&ops).erases, oracle::erase_ops(&ops));
    }
}

/// Guarantee 3: an erase that arrives before its Add, directly or through `merge`.
#[test]
fn erase_before_add() {
    let stroke = add(id(0, 0), 1);
    let erasure = erase(id(0, 1), id(0, 0));

    let mut board = Board::new();
    assert!(!board.apply(erasure.clone()));
    assert!(!board.apply(stroke.clone()));
    assert_eq!(board.picture(), []);

    let (only_erase, only_add) = (Board::of([&erasure]), Board::of([&stroke]));
    assert_eq!(only_erase.merged(&only_add).picture(), []);
    assert_eq!(only_add.merged(&only_erase).picture(), []);
}

/// Guarantee 5: an erase target whose Add has not arrived is an id `draw` must not reuse, or the
/// new stroke would be born erased.
#[test]
fn draw_skips_ids_of_erase_targets() {
    let me = PeerId(1);
    let target = id(1, 2);
    let mut board = Board::of([&erase(id(0, 0), target)]);
    for _ in 0..3 {
        let stroke = board.draw_any(me);
        assert_ne!(stroke.id, target);
        assert_eq!(board.visible().last(), Some(&stroke));
    }
}

/// Foreign ops are not validated: counters already at `u64::MAX` saturate instead of panicking.
#[test]
fn saturated_foreign_counters_do_not_panic() {
    let me = PeerId(1);
    let mut board = Board::of([&add(id(1, u64::MAX), u64::MAX)]);
    board.draw_any(me);
    let _ = board.erase(me, id(0, 0));
}

/// Guarantee 7: an erase whose target never arrived is kept with its op id and survives `merge`.
#[test]
fn retains_erase_op_before_add() {
    let board = Board::new().merged(&Board::of([&erase(id(1, 4), id(0, 9))]));
    assert_eq!(board.erases, BTreeMap::from([(id(1, 4), id(0, 9))]));
}

/// Guarantee 6: erase op ids are not hashed; an added id and a tombstoned target hash apart.
#[test]
fn state_hash_ignores_erase_op_ids() {
    let target = id(0, 0);
    let by_one = Board::of([&erase(id(1, 0), target)]);
    let by_other = Board::of([&erase(id(2, 0), target)]);
    assert_eq!(by_one.state_hash(), by_other.state_hash());

    let added = Board::of([&add(target, 1)]);
    assert_ne!(added.state_hash(), by_one.state_hash());
}
