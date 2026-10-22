//! Conformance tests for `docs/contracts/board.md`; each test names the guarantee it checks.

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use super::{ADDED, Board, TOMBSTONED, element_hash};
use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};

const ORIGIN: WorldPos = WorldPos { x: 0, y: 0 };

/// Op `i` of a generated universe; ids are unique and spread over 3 peers.
fn op_id(i: u64) -> OpId {
    OpId {
        peer: PeerId(i % 3),
        seq: i,
    }
}

fn stroke(id: OpId, lamport: u64) -> Stroke {
    Stroke {
        id,
        lamport: Lamport(lamport),
        color: Rgba([0, 0, 0, 255]),
        width: 16,
        bbox: Aabb {
            min: ORIGIN,
            max: ORIGIN,
        },
        points: EncodedPoints(Vec::new()),
    }
}

fn draw(board: &mut Board, me: PeerId) -> Op {
    let bbox = Aabb {
        min: ORIGIN,
        max: ORIGIN,
    };
    board.draw(
        me,
        Rgba([255, 0, 0, 255]),
        32,
        EncodedPoints(vec![1, 2]),
        bbox,
    )
}

/// A universe of ops with unique ids. Lamports collide on purpose; an erase targets any op id of
/// the universe, a later one, or one that never exists.
fn ops() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec((any::<bool>(), 0..4u64, 0..48u64), 0..40).prop_map(|specs| {
        (0u64..)
            .zip(specs)
            .map(|(i, (is_add, lamport, target))| {
                if is_add {
                    Op::Add(stroke(op_id(i), lamport))
                } else {
                    Op::Erase {
                        id: op_id(i),
                        target: op_id(target),
                    }
                }
            })
            .collect()
    })
}

/// An op universe and the same ops shuffled.
fn permutations() -> impl Strategy<Value = (Vec<Op>, Vec<Op>)> {
    ops().prop_flat_map(|ops| (Just(ops.clone()), Just(ops).prop_shuffle()))
}

/// Three replicas, each holding a random subset of one op universe.
fn replicas() -> impl Strategy<Value = [Board; 3]> {
    (ops(), prop::collection::vec(any::<u8>(), 40)).prop_map(|(ops, masks)| {
        std::array::from_fn(|r| {
            board_of(
                ops.iter()
                    .zip(&masks)
                    .filter(|(_, mask)| *mask & (1 << r) != 0)
                    .map(|(op, _)| op),
            )
        })
    })
}

fn board_of<'a>(ops: impl IntoIterator<Item = &'a Op>) -> Board {
    let mut board = Board::new();
    for op in ops {
        board.apply(op.clone());
    }
    board
}

fn merged(a: &Board, b: &Board) -> Board {
    let mut board = a.clone();
    board.merge(b);
    board
}

fn picture(board: &Board) -> Vec<Stroke> {
    board.visible().cloned().collect()
}

/// `state_hash` recomputed from scratch over the canonical state.
fn canonical_hash(board: &Board) -> u128 {
    let added = board.strokes.keys().map(|&id| element_hash(ADDED, id));
    let tombstoned = board
        .tombstones
        .iter()
        .map(|&id| element_hash(TOMBSTONED, id));
    added.chain(tombstoned).fold(0, u128::wrapping_add)
}

proptest! {
    /// Guarantee 1.
    #[test]
    fn merge_laws([a, b, c] in replicas()) {
        prop_assert_eq!(picture(&merged(&a, &b)), picture(&merged(&b, &a)));
        prop_assert_eq!(
            picture(&merged(&merged(&a, &b), &c)),
            picture(&merged(&a, &merged(&b, &c)))
        );
        prop_assert_eq!(picture(&merged(&a, &a)), picture(&a));
    }

    /// Guarantee 2; `apply` reports exactly when the picture changes.
    #[test]
    fn apply_any_permutation((ops, shuffled) in permutations()) {
        let mut board = Board::new();
        for op in &shuffled {
            let before = picture(&board);
            let changed = board.apply(op.clone());
            prop_assert_eq!(changed, picture(&board) != before);
        }
        prop_assert_eq!(picture(&board), picture(&board_of(&ops)));
        for op in ops {
            prop_assert!(!board.apply(op));
        }
        prop_assert_eq!(picture(&board), picture(&board_of(&shuffled)));
    }

    /// Guarantees 3 and 4: `visible` equals the unerased adds, sorted by `(lamport, peer)`.
    #[test]
    fn visible_is_sorted(ops in ops()) {
        let erased: BTreeSet<OpId> = ops
            .iter()
            .filter_map(|op| match op {
                Op::Erase { target, .. } => Some(*target),
                Op::Add(_) => None,
            })
            .collect();
        let mut expected: Vec<Stroke> = ops
            .iter()
            .filter_map(|op| match op {
                Op::Add(stroke) if !erased.contains(&stroke.id) => Some(stroke.clone()),
                _ => None,
            })
            .collect();
        expected.sort_by_key(|stroke| (stroke.lamport, stroke.id));

        let board = board_of(&ops);
        prop_assert!(board.visible().map(|s| (s.lamport, s.id.peer)).is_sorted());
        prop_assert_eq!(picture(&board), expected);
    }

    /// Guarantee 5.
    #[test]
    fn draw_after_merge_is_on_top([a, b, _] in replicas()) {
        let seen_lamport = a.strokes.values().chain(b.strokes.values()).map(|s| s.lamport).max();
        let seen_ids: BTreeSet<OpId> = [&a, &b]
            .into_iter()
            .flat_map(|board| board.strokes.keys().chain(board.erases.keys()))
            .copied()
            .collect();

        let mut board = merged(&a, &b);
        let me = PeerId(1);
        let Op::Add(first) = draw(&mut board, me) else { panic!("draw returns an Add") };
        prop_assert!(Some(first.lamport) > seen_lamport);
        prop_assert_eq!(board.visible().last(), Some(&first));

        let Op::Erase { id: erase_id, .. } = board.erase(me, first.id) else {
            panic!("erase returns an Erase")
        };
        let Op::Add(second) = draw(&mut board, me) else { panic!("draw returns an Add") };
        prop_assert!(second.lamport > first.lamport);
        let mine = [first.id, erase_id, second.id];
        prop_assert_eq!(mine.iter().collect::<BTreeSet<_>>().len(), mine.len());
        prop_assert!(mine.iter().all(|id| !seen_ids.contains(id)));
    }

    /// Guarantee 6.
    #[test]
    fn state_hash_tracks_canonical_state_and_matches_equal_boards((ops, shuffled) in permutations()) {
        let board = board_of(&ops);
        prop_assert_eq!(board.state_hash(), canonical_hash(&board));
        prop_assert_eq!(board.state_hash(), board_of(&shuffled).state_hash());

        // Duplicate adds and repeated erases (with new erase OpIds) leave it unchanged.
        let mut again = board.clone();
        for stroke in board.strokes.values() {
            again.apply(Op::Add(stroke.clone()));
        }
        for &target in &board.tombstones {
            again.erase(PeerId(7), target);
        }
        prop_assert_eq!(again.state_hash(), board.state_hash());

        // A new stroke id and a new tombstone each change it.
        let Op::Add(fresh) = draw(&mut again, PeerId(7)) else { panic!("draw returns an Add") };
        let with_stroke = again.state_hash();
        prop_assert_ne!(with_stroke, board.state_hash());
        again.erase(PeerId(7), fresh.id);
        prop_assert_ne!(again.state_hash(), with_stroke);
        prop_assert_eq!(again.state_hash(), canonical_hash(&again));
    }

    /// Guarantee 7.
    #[test]
    fn retains_erase_ops_with_op_id_and_target(ops in ops()) {
        let expected: BTreeMap<OpId, OpId> = ops
            .iter()
            .filter_map(|op| match op {
                Op::Erase { id, target } => Some((*id, *target)),
                Op::Add(_) => None,
            })
            .collect();
        prop_assert_eq!(&board_of(&ops).erases, &expected);
    }
}

/// Guarantee 3: an erase that arrives before its Add, directly or through `merge`.
#[test]
fn erase_before_add() {
    let add = Op::Add(stroke(op_id(0), 1));
    let erase = Op::Erase {
        id: op_id(1),
        target: op_id(0),
    };

    let mut board = Board::new();
    assert!(!board.apply(erase.clone()));
    assert!(!board.apply(add.clone()));
    assert_eq!(board.visible().count(), 0);

    let only_erase = board_of([&erase]);
    let only_add = board_of([&add]);
    assert_eq!(merged(&only_erase, &only_add).visible().count(), 0);
    assert_eq!(merged(&only_add, &only_erase).visible().count(), 0);
}

/// Guarantee 7: an erase whose target never arrived is kept with its op id and survives `merge`.
#[test]
fn retains_erase_op_before_add() {
    let erase = Op::Erase {
        id: op_id(4),
        target: op_id(9),
    };
    let board = merged(&Board::new(), &board_of([&erase]));
    assert_eq!(board.erases, BTreeMap::from([(op_id(4), op_id(9))]));
}

/// Guarantee 6: erase op ids are not hashed; an added id and a tombstoned target hash apart.
#[test]
fn state_hash_ignores_erase_op_ids() {
    let target = op_id(0);
    let by_one = board_of([&Op::Erase {
        id: op_id(1),
        target,
    }]);
    let by_other = board_of([&Op::Erase {
        id: op_id(2),
        target,
    }]);
    assert_eq!(by_one.state_hash(), by_other.state_hash());

    let added = board_of([&Op::Add(stroke(target, 1))]);
    assert_ne!(added.state_hash(), by_one.state_hash());
    assert_eq!(Board::new().state_hash(), 0);
}
