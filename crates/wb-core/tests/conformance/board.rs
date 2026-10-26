//! The `Board` suite: one check per guarantee of the trait, run on every impl below.

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;
use wb_core::{Board, Op, OpId, PeerId};

use crate::support::board::{BoardExt, OpExt, OpIdExt, oracle};

/// Guarantee 1, plus equal `state_hash` for merged boards with equal state (guarantee 6).
fn merge_laws<B: Board>(ops: &[Vec<Op>; 3]) -> Result<(), TestCaseError> {
    let [a, b, c] = ops.each_ref().map(|ops| B::of(ops));
    let ab = a.merged(&b);
    let ba = b.merged(&a);
    prop_assert_eq!(ab.picture(), ba.picture());
    prop_assert_eq!(ab.merged(&c).picture(), a.merged(&b.merged(&c)).picture());
    prop_assert_eq!(a.merged(&a).picture(), a.picture());
    let mut twice = B::of(&ops[0]);
    twice.merge(&a);
    prop_assert_eq!(twice.picture(), a.picture());

    prop_assert_eq!(ab.state_hash(), ba.state_hash());
    prop_assert_eq!(ab.state_hash(), B::of(&ops[..2].concat()).state_hash());
    Ok(())
}

/// Guarantee 2; `apply` also reports exactly when the picture changes.
fn apply_any_permutation<B: Board>(ops: &[Op], delivered: &[Op]) -> Result<(), TestCaseError> {
    let mut board = B::default();
    for op in delivered {
        let before = board.picture();
        let changed = board.apply(op.clone());
        prop_assert_eq!(changed, board.picture() != before);
    }
    let settled = board.picture();
    prop_assert_eq!(&settled, &B::of(ops).picture());
    for op in ops {
        prop_assert!(!board.apply(op.clone()));
    }
    prop_assert_eq!(board.picture(), settled);
    Ok(())
}

/// Guarantee 3: an erase that arrives before its Add, directly or through `merge`.
fn erase_before_add<B: Board>() {
    let stroke = Op::add(OpId::of(0, 0), 1);
    let erasure = Op::erase(OpId::of(0, 1), OpId::of(0, 0));

    let mut board = B::default();
    assert!(!board.apply(erasure.clone()));
    assert!(!board.apply(stroke.clone()));
    assert_eq!(board.picture(), []);

    let (only_erase, only_add) = (B::of(&[erasure]), B::of(&[stroke]));
    assert_eq!(only_erase.merged(&only_add).picture(), []);
    assert_eq!(only_add.merged(&only_erase).picture(), []);
}

/// Guarantees 3 and 4: `visible` is exactly the unerased adds, sorted by `(lamport, peer)`.
fn visible_is_sorted<B: Board>(ops: &[Op]) -> Result<(), TestCaseError> {
    let board = B::of(ops);
    prop_assert!(board.visible().map(|s| (s.lamport, s.id.peer)).is_sorted());
    prop_assert_eq!(board.picture(), oracle::picture(ops));
    Ok(())
}

/// Guarantee 5: above every Lamport seen, and no id seen as an op or as an erase target.
fn draw_after_merge_is_on_top<B: Board>(a: &[Op], b: &[Op]) -> Result<(), TestCaseError> {
    let seen = [a, b].concat();
    let mut board = B::of(a).merged(&B::of(b));
    let me = PeerId(1);

    let first = board.draw_any(me);
    prop_assert!(Some(first.lamport) > oracle::max_lamport(&seen));
    prop_assert_eq!(board.visible().last(), Some(&first));

    let erased = board.erase(me, first.id);
    let second = board.draw_any(me);
    prop_assert!(second.lamport > first.lamport);

    let mine = [first.id, erased.id(), second.id];
    let seen_ids = oracle::ids(&seen);
    prop_assert_eq!(BTreeSet::from(mine).len(), mine.len());
    prop_assert!(mine.iter().all(|id| !seen_ids.contains(id)));
    Ok(())
}

/// Guarantee 5: an erase target whose Add has not arrived is an id `draw` must not reuse, or the
/// new stroke would be born erased.
fn draw_skips_ids_of_erase_targets<B: Board>() {
    let target = OpId::of(1, 0);
    let mut board = B::of(&[Op::erase(OpId::of(0, 0), target)]);
    let stroke = board.draw_any(PeerId(1));
    assert_ne!(stroke.id, target);
    assert_eq!(board.visible().last(), Some(&stroke));
}

/// Guarantee 6: the hash is a function of the canonical state alone, with each item once, and
/// erase op ids not hashed.
fn state_hash_tracks_canonical_state_and_matches_equal_boards<B: Board>(
    ops: &[Op],
    delivered: &[Op],
) -> Result<(), TestCaseError> {
    let mut board = B::of(ops);
    let hash = board.state_hash();
    prop_assert_eq!(hash, B::of(&oracle::canonical(ops)).state_hash());
    prop_assert_eq!(hash, B::of(delivered).state_hash());

    // Duplicate adds, and a second erase of every target by a new op id, change nothing.
    let mut log = ops.to_vec();
    for op in ops {
        match op {
            Op::Add(_) => {
                board.apply(op.clone());
            }
            Op::Erase { target, .. } => log.push(board.erase(PeerId(7), *target)),
        }
    }
    prop_assert_eq!(board.state_hash(), hash);

    // A new stroke id, then a new tombstone, each change it.
    let fresh = board.draw_any(PeerId(7));
    log.push(Op::Add(fresh.clone()));
    let with_stroke = board.state_hash();
    prop_assert_ne!(with_stroke, hash);
    log.push(board.erase(PeerId(7), fresh.id));
    prop_assert_ne!(board.state_hash(), with_stroke);
    prop_assert_eq!(
        board.state_hash(),
        B::of(&oracle::canonical(&log)).state_hash()
    );
    Ok(())
}

/// Guarantee 7, through `apply` and through `merge`, including erases whose target never
/// arrives and several erases of one target.
fn retains_erase_ops_with_op_id_and_target<B: Board>(
    a: &[Op],
    b: &[Op],
) -> Result<(), TestCaseError> {
    let union = [a, b].concat();
    let expected = oracle::erase_ops(&union);
    let held = |board: &B| board.erase_ops().collect::<BTreeMap<_, _>>();
    prop_assert_eq!(&held(&B::of(&union)), &expected);
    prop_assert_eq!(&held(&B::of(a).merged(&B::of(b))), &expected);
    prop_assert_eq!(B::of(&union).erase_ops().count(), expected.len());
    Ok(())
}

mod tombstone_board {
    use proptest::prelude::*;
    use wb_core::TombstoneBoard;

    use crate::support::board::strategy;

    type B = TombstoneBoard;

    proptest! {
        #[test]
        fn merge_laws(ops in strategy::replicas()) {
            super::merge_laws::<B>(&ops)?;
        }

        #[test]
        fn apply_any_permutation((ops, delivered) in strategy::permutations()) {
            super::apply_any_permutation::<B>(&ops, &delivered)?;
        }

        #[test]
        fn visible_is_sorted(ops in strategy::ops()) {
            super::visible_is_sorted::<B>(&ops)?;
        }

        #[test]
        fn draw_after_merge_is_on_top([a, b, _] in strategy::replicas()) {
            super::draw_after_merge_is_on_top::<B>(&a, &b)?;
        }

        #[test]
        fn state_hash_tracks_canonical_state_and_matches_equal_boards(
            (ops, delivered) in strategy::permutations(),
        ) {
            super::state_hash_tracks_canonical_state_and_matches_equal_boards::<B>(
                &ops, &delivered,
            )?;
        }

        #[test]
        fn retains_erase_ops_with_op_id_and_target([a, b, _] in strategy::replicas()) {
            super::retains_erase_ops_with_op_id_and_target::<B>(&a, &b)?;
        }
    }

    #[test]
    fn erase_before_add() {
        super::erase_before_add::<B>();
    }

    #[test]
    fn draw_skips_ids_of_erase_targets() {
        super::draw_skips_ids_of_erase_targets::<B>();
    }
}
