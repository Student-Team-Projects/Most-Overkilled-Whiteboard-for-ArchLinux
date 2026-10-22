//! Conformance tests for `docs/contracts/board.md`; each test names the guarantee it checks.

use std::collections::{BTreeMap, BTreeSet};

use proptest::prelude::*;

use super::{ADDED, Board, TOMBSTONED, element_hash};
use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};

/// Most ops a generated universe holds.
const UNIVERSE: usize = 40;

const BBOX: Aabb = Aabb {
    min: WorldPos { x: 0, y: 0 },
    max: WorldPos { x: 0, y: 0 },
};

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
        bbox: BBOX,
        points: EncodedPoints(Vec::new()),
    }
}

fn draw(board: &mut Board, me: PeerId) -> Stroke {
    let op = board.draw(
        me,
        Rgba([255, 0, 0, 255]),
        32,
        EncodedPoints(vec![1, 2]),
        BBOX,
    );
    let Op::Add(stroke) = op else {
        panic!("draw returns an Add")
    };
    stroke
}

/// A universe of ops with unique ids. Lamports collide on purpose; an erase targets any op id of
/// the universe (an Add, an Erase, a later op) or one that never exists.
fn ops() -> impl Strategy<Value = Vec<Op>> {
    prop::collection::vec((any::<bool>(), 0..4u64, 0..48u64), 0..UNIVERSE).prop_map(|specs| {
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

/// The ops of three replicas, each a random subset of one universe.
fn replicas() -> impl Strategy<Value = [Vec<Op>; 3]> {
    (ops(), prop::collection::vec(any::<u8>(), UNIVERSE)).prop_map(|(ops, masks)| {
        std::array::from_fn(|r| {
            ops.iter()
                .zip(&masks)
                .filter(|(_, mask)| *mask & (1 << r) != 0)
                .map(|(op, _)| op.clone())
                .collect()
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

/// An empty board with `a`, then `b`, merged into it.
fn merged(a: &Board, b: &Board) -> Board {
    let mut board = Board::new();
    board.merge(a);
    board.merge(b);
    board
}

fn picture(board: &Board) -> Vec<Stroke> {
    board.visible().cloned().collect()
}

/// The expected `state_hash`, from the ops alone: the added ids plus the erase targets.
fn canonical_hash<'a>(ops: impl IntoIterator<Item = &'a Op>) -> u128 {
    let (mut added, mut tombstoned) = (BTreeSet::new(), BTreeSet::new());
    for op in ops {
        match op {
            Op::Add(stroke) => added.insert(stroke.id),
            Op::Erase { target, .. } => tombstoned.insert(*target),
        };
    }
    let added = added.into_iter().map(|id| element_hash(ADDED, id));
    let tombstoned = tombstoned
        .into_iter()
        .map(|id| element_hash(TOMBSTONED, id));
    added.chain(tombstoned).fold(0, u128::wrapping_add)
}

proptest! {
    /// Guarantee 1, plus a merged board's `state_hash` (guarantee 6).
    #[test]
    fn merge_laws([a, b, c] in replicas()) {
        let (a_ops, b_ops) = (a.clone(), b.clone());
        let [a, b, c] = [a, b, c].map(|ops| board_of(&ops));
        let ab = merged(&a, &b);
        prop_assert_eq!(picture(&ab), picture(&merged(&b, &a)));
        prop_assert_eq!(
            picture(&merged(&ab, &c)),
            picture(&merged(&a, &merged(&b, &c)))
        );
        prop_assert_eq!(picture(&merged(&a, &a)), picture(&a));

        prop_assert_eq!(ab.state_hash(), merged(&b, &a).state_hash());
        prop_assert_eq!(ab.state_hash(), canonical_hash(a_ops.iter().chain(&b_ops)));
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
        let settled = picture(&board);
        prop_assert_eq!(&settled, &picture(&board_of(&ops)));
        for op in ops {
            prop_assert!(!board.apply(op));
        }
        prop_assert_eq!(picture(&board), settled);
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

    /// Guarantee 5: above every Lamport seen, and no id seen as an op or an erase target.
    #[test]
    fn draw_after_merge_is_on_top([a, b, _] in replicas()) {
        let seen = || a.iter().chain(&b);
        let seen_lamport = seen()
            .filter_map(|op| match op {
                Op::Add(stroke) => Some(stroke.lamport),
                Op::Erase { .. } => None,
            })
            .max();
        let seen_ids: BTreeSet<OpId> = seen()
            .flat_map(|op| match op {
                Op::Add(stroke) => vec![stroke.id],
                Op::Erase { id, target } => vec![*id, *target],
            })
            .collect();

        let mut board = merged(&board_of(&a), &board_of(&b));
        let me = PeerId(1);
        let first = draw(&mut board, me);
        prop_assert!(Some(first.lamport) > seen_lamport);
        prop_assert_eq!(board.visible().last(), Some(&first));

        let Op::Erase { id: erase_id, .. } = board.erase(me, first.id) else {
            panic!("erase returns an Erase")
        };
        let second = draw(&mut board, me);
        prop_assert!(second.lamport > first.lamport);
        let mine = [first.id, erase_id, second.id];
        prop_assert_eq!(mine.iter().collect::<BTreeSet<_>>().len(), mine.len());
        prop_assert!(mine.iter().all(|id| !seen_ids.contains(id)));
    }

    /// Guarantee 6.
    #[test]
    fn state_hash_tracks_canonical_state_and_matches_equal_boards((ops, shuffled) in permutations()) {
        let mut board = board_of(&ops);
        let hash = board.state_hash();
        prop_assert_eq!(hash, canonical_hash(&ops));
        prop_assert_eq!(hash, board_of(&shuffled).state_hash());

        // Duplicate adds, and a second erase of every target by a new erase op id, change nothing.
        let mut log = ops.clone();
        for op in &ops {
            match op {
                Op::Add(_) => prop_assert!(!board.apply(op.clone())),
                Op::Erase { target, .. } => log.push(board.erase(PeerId(7), *target)),
            }
        }
        prop_assert_eq!(board.state_hash(), hash);

        // A new stroke id, then a new tombstone, each change it.
        let fresh = draw(&mut board, PeerId(7));
        let with_stroke = board.state_hash();
        prop_assert_ne!(with_stroke, hash);
        log.push(board.erase(PeerId(7), fresh.id));
        log.push(Op::Add(fresh));
        prop_assert_ne!(board.state_hash(), with_stroke);
        prop_assert_eq!(board.state_hash(), canonical_hash(&log));
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

/// Guarantee 5: an erase target whose Add has not arrived is an id `draw` must not reuse, or the
/// new stroke would be born erased.
#[test]
fn draw_skips_ids_of_erase_targets() {
    let me = PeerId(1);
    let target = OpId { peer: me, seq: 2 };
    let mut board = board_of([&Op::Erase {
        id: OpId {
            peer: PeerId(0),
            seq: 0,
        },
        target,
    }]);
    for _ in 0..3 {
        let stroke = draw(&mut board, me);
        assert_ne!(stroke.id, target);
        assert_eq!(board.visible().last(), Some(&stroke));
    }
}

/// Foreign ops are not validated: counters already at `u64::MAX` saturate instead of panicking.
#[test]
fn saturated_foreign_counters_do_not_panic() {
    let me = PeerId(1);
    let mut board = board_of([&Op::Add(stroke(
        OpId {
            peer: me,
            seq: u64::MAX,
        },
        u64::MAX,
    ))]);
    let _ = draw(&mut board, me);
    let _ = board.erase(me, op_id(0));
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
}
