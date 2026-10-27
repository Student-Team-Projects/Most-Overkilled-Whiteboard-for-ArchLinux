//! Merge, apply, erase and the order of visible strokes.

use proptest::prelude::*;
use wb_core::{Board, Op, OpId};

use crate::support::board::{BoardExt, OpExt, OpIdExt, oracle, strategy};

proptest! {
    #[test]
    fn merge_laws(ops in strategy::replicas()) {
        let [a, b, c] = ops.each_ref().map(|ops| Board::of(ops));
        let ab = a.merged(&b);
        let ba = b.merged(&a);
        prop_assert_eq!(ab.picture(), ba.picture());
        prop_assert_eq!(ab.merged(&c).picture(), a.merged(&b.merged(&c)).picture());
        prop_assert_eq!(a.merged(&a).picture(), a.picture());
        let mut twice = Board::of(&ops[0]);
        twice.merge(&a);
        prop_assert_eq!(twice.picture(), a.picture());

        prop_assert_eq!(ab.state_hash(), ba.state_hash());
        prop_assert_eq!(ab.state_hash(), Board::of(&ops[..2].concat()).state_hash());
    }

    #[test]
    fn apply_any_permutation((ops, delivered) in strategy::permutations()) {
        let mut board = Board::new();
        for op in &delivered {
            let before = board.picture();
            let changed = board.apply(op.clone());
            prop_assert_eq!(changed, board.picture() != before);
        }
        let settled = board.picture();
        prop_assert_eq!(&settled, &Board::of(&ops).picture());
        for op in &ops {
            prop_assert!(!board.apply(op.clone()));
        }
        prop_assert_eq!(board.picture(), settled);
    }

    #[test]
    fn visible_is_sorted(ops in strategy::ops()) {
        let board = Board::of(&ops);
        prop_assert!(board.visible().map(|s| (s.lamport, s.id.peer)).is_sorted());
        prop_assert_eq!(board.picture(), oracle::picture(&ops));
    }
}

#[test]
fn erase_before_add() {
    let stroke = Op::add(OpId::of(0, 0), 1);
    let erasure = Op::erase(OpId::of(0, 1), OpId::of(0, 0));

    let mut board = Board::new();
    assert!(!board.apply(erasure.clone()));
    assert!(!board.apply(stroke.clone()));
    assert_eq!(board.picture(), []);

    let (only_erase, only_add) = (Board::of(&[erasure]), Board::of(&[stroke]));
    assert_eq!(only_erase.merged(&only_add).picture(), []);
    assert_eq!(only_add.merged(&only_erase).picture(), []);
}
