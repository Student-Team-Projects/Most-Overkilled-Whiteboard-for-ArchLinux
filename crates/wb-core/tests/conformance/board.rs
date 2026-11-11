//! `Board` laws, generic over the board; `board_conformance!` runs them on one.

use std::collections::BTreeSet;

use proptest::prelude::*;
use wb_core::{Board, Op, OpId, PeerId, TombstoneBoard};

use crate::support::board::{BoardExt, OpExt, OpIdExt, oracle};
use crate::support::config::{ME, PAGE_PITCH};

/// Generates a module of tests that run every law on `$board`.
macro_rules! board_conformance {
    ($impl:ident, $board:ty) => {
        mod $impl {
            use super::*;
            use crate::support::board::strategy;

            proptest! {
                #[test]
                fn merge_laws(ops in strategy::replicas()) {
                    super::merge_laws::<$board>(&ops)?;
                }

                #[test]
                fn apply_any_permutation((ops, delivered) in strategy::permutations()) {
                    super::apply_any_permutation::<$board>(&ops, &delivered)?;
                }

                #[test]
                fn visible_is_sorted(ops in strategy::ops()) {
                    super::visible_is_sorted::<$board>(&ops)?;
                }

                #[test]
                fn stamp_after_merge_is_on_top(ops in strategy::replicas()) {
                    super::stamp_after_merge_is_on_top::<$board>(&ops)?;
                }

                #[test]
                fn page_count_is_max_of_add_pages_and_stroke_pages(
                    (ops, delivered) in strategy::permutations(),
                ) {
                    super::page_count_is_max_of_add_pages_and_stroke_pages::<$board>(
                        &ops, &delivered,
                    )?;
                }

                #[test]
                fn state_hash_tracks_canonical_state_and_matches_equal_boards(
                    (ops, delivered) in strategy::permutations(),
                ) {
                    super::state_hash_tracks_canonical_state_and_matches_equal_boards::<$board>(
                        &ops, &delivered,
                    )?;
                }
            }

            #[test]
            fn erase_before_add() {
                super::erase_before_add::<$board>();
            }

            #[test]
            fn stamp_skips_ids_of_erase_targets() {
                super::stamp_skips_ids_of_erase_targets::<$board>();
            }

            #[test]
            fn unapplied_stamps_never_repeat() {
                super::unapplied_stamps_never_repeat::<$board>();
            }

            #[test]
            fn concurrent_adds_of_one_page_give_one_page() {
                super::concurrent_adds_of_one_page_give_one_page::<$board>();
            }

            #[test]
            fn a_stroke_counts_the_page_that_holds_its_lowest_point() {
                super::a_stroke_counts_the_page_that_holds_its_lowest_point::<$board>();
            }

            #[test]
            fn the_most_pages_asked_for_is_u32_max() {
                super::the_most_pages_asked_for_is_u32_max::<$board>();
            }
        }
    };
}

board_conformance!(tombstone_board, TombstoneBoard);

fn merge_laws<B: Board>(ops: &[Vec<Op>; 3]) -> Result<(), TestCaseError> {
    let [a, b, c] = ops.each_ref().map(|ops| B::of(ops));
    let ab = a.merged(&b);
    let ba = b.merged(&a);
    prop_assert_eq!(ab.view(), ba.view());
    prop_assert_eq!(ab.merged(&c).view(), a.merged(&b.merged(&c)).view());
    prop_assert_eq!(a.merged(&a).view(), a.view());
    let mut twice = B::of(&ops[0]);
    twice.merge(&a);
    prop_assert_eq!(twice.view(), a.view());
    prop_assert_eq!(ab.view(), B::of(&ops[..2].concat()).view());

    prop_assert_eq!(ab.state_hash(), ba.state_hash());
    prop_assert_eq!(ab.state_hash(), B::of(&ops[..2].concat()).state_hash());
    Ok(())
}

fn apply_any_permutation<B: Board>(ops: &[Op], delivered: &[Op]) -> Result<(), TestCaseError> {
    let mut board = B::default();
    for op in delivered {
        let before = board.view();
        let changed = board.apply(op.clone());
        prop_assert_eq!(changed, board.view() != before);
    }
    let settled = board.view();
    prop_assert_eq!(&settled, &B::of(ops).view());
    for op in ops {
        prop_assert!(!board.apply(op.clone()));
    }
    prop_assert_eq!(board.view(), settled);
    Ok(())
}

fn erase_before_add<B: Board>() {
    let stroke = Op::stroke(OpId::of(0, 0), 1);
    let erasure = Op::erasure(OpId::of(0, 1), OpId::of(0, 0));

    let mut board = B::default();
    assert!(!board.apply(erasure.clone()));
    assert!(!board.apply(stroke.clone()));
    assert_eq!(board.picture(), []);

    let (only_erase, only_add) = (B::of(&[erasure]), B::of(&[stroke]));
    assert_eq!(only_erase.merged(&only_add).picture(), []);
    assert_eq!(only_add.merged(&only_erase).picture(), []);
}

fn visible_is_sorted<B: Board>(ops: &[Op]) -> Result<(), TestCaseError> {
    let board = B::of(ops);
    prop_assert!(board.visible().map(|s| (s.lamport, s.id.peer)).is_sorted());
    prop_assert_eq!(board.picture(), oracle::picture(ops));
    Ok(())
}

fn stamp_after_merge_is_on_top<B: Board>([a, b, _]: &[Vec<Op>; 3]) -> Result<(), TestCaseError> {
    let seen = [&a[..], &b[..]].concat();
    let mut board = B::of(a).merged(&B::of(b));
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
    Ok(())
}

/// An erase target whose Add has not arrived is an id `stamp` must not reuse, or the new stroke
/// would be born erased.
fn stamp_skips_ids_of_erase_targets<B: Board>() {
    let target = OpId::new(ME, 0);
    let mut board = B::of(&[Op::erasure(OpId::of(0, 0), target)]);
    let stroke = board.draw_any(ME);
    assert_ne!(stroke.id, target);
    assert_eq!(board.visible().last(), Some(&stroke));
}

/// A stamp whose op is dropped is still spent: the next one has a new id and a higher Lamport.
fn unapplied_stamps_never_repeat<B: Board>() {
    let mut board = B::default();
    let first = board.stamp(ME);
    let second = board.stamp(ME);
    assert_ne!(first.id, second.id);
    assert!(second.lamport > first.lamport);
}

fn state_hash_tracks_canonical_state_and_matches_equal_boards<B: Board>(
    ops: &[Op],
    delivered: &[Op],
) -> Result<(), TestCaseError> {
    let mut board = B::of(ops);
    let hash = board.state_hash();
    prop_assert_eq!(hash, B::of(&oracle::canonical(ops)).state_hash());
    prop_assert_eq!(hash, B::of(delivered).state_hash());

    // Duplicate adds, a second erase of every target and a second ask for every page count, by
    // new op ids, change nothing.
    let mut log = ops.to_vec();
    for op in ops {
        match op {
            Op::Add(_) => {
                board.apply(op.clone());
            }
            Op::Erase { target, .. } => log.push(board.erase(ME, *target)),
            Op::AddPage { count, .. } => log.push(board.add_pages(ME, *count)),
        }
    }
    prop_assert_eq!(board.state_hash(), hash);

    // One more page changes it.
    log.push(board.add_pages(ME, board.page_count().saturating_add(1)));
    let with_page = board.state_hash();
    prop_assert_ne!(with_page, hash);
    let hash = with_page;

    // A new stroke id, then a new tombstone, each change it.
    let fresh = board.draw_any(ME);
    log.push(Op::Add(fresh.clone()));
    let with_stroke = board.state_hash();
    prop_assert_ne!(with_stroke, hash);
    log.push(board.erase(ME, fresh.id));
    prop_assert_ne!(board.state_hash(), with_stroke);
    prop_assert_eq!(
        board.state_hash(),
        B::of(&oracle::canonical(&log)).state_hash()
    );
    Ok(())
}

/// Never below 1, never decreasing while ops arrive in any order and twice, and in the end what
/// the ops ask for: the largest add-page count, or one past the lowest stroke's page.
fn page_count_is_max_of_add_pages_and_stroke_pages<B: Board>(
    ops: &[Op],
    delivered: &[Op],
) -> Result<(), TestCaseError> {
    let mut board = B::default();
    prop_assert_eq!(board.page_count(), 1);
    for op in delivered {
        let before = board.page_count();
        board.apply(op.clone());
        prop_assert!(
            board.page_count() >= before,
            "the page count never decreases"
        );
    }
    prop_assert_eq!(board.page_count(), oracle::page_count(ops));
    Ok(())
}

/// Two peers that each add a page at the end of the same board, before hearing of each other,
/// end with one new page.
fn concurrent_adds_of_one_page_give_one_page<B: Board>() {
    let shared = B::of(&[Op::page_add(OpId::of(0, 0), 3)]);
    let mut a = B::default().merged(&shared);
    let mut b = B::default().merged(&shared);
    a.add_pages(PeerId(1), a.page_count().saturating_add(1));
    b.add_pages(PeerId(2), b.page_count().saturating_add(1));
    assert_eq!(a.merged(&b).page_count(), 4);
}

/// A page owns the gap below it; a stroke above the first page still leaves one page, and one
/// past the last page saturates.
fn a_stroke_counts_the_page_that_holds_its_lowest_point<B: Board>() {
    let pages = |y| B::of(&[Op::stroke_down_to(OpId::of(0, 0), 0, y)]).page_count();
    assert_eq!(pages(0), 1);
    assert_eq!(pages(PAGE_PITCH - 1), 1, "the gap below page 0");
    assert_eq!(pages(PAGE_PITCH), 2);
    assert_eq!(pages(-1), 1);
    assert_eq!(pages(i64::MIN), 1);
    assert_eq!(pages(i64::MAX), u32::MAX);
    let last_page_top = i64::from(u32::MAX) * PAGE_PITCH;
    assert_eq!(pages(last_page_top - 1), u32::MAX);
    assert_eq!(
        pages(last_page_top),
        u32::MAX,
        "one past the last page saturates"
    );

    let erased = B::of(&[
        Op::stroke_down_to(OpId::of(0, 0), 0, 2 * PAGE_PITCH),
        Op::erasure(OpId::of(0, 1), OpId::of(0, 0)),
    ]);
    assert_eq!(erased.page_count(), 3, "an erased stroke keeps its page");
}

/// Asking for every page there can be works, and asking again changes nothing.
fn the_most_pages_asked_for_is_u32_max<B: Board>() {
    let mut board = B::of(&[Op::page_add(OpId::of(0, 0), u32::MAX)]);
    assert_eq!(board.page_count(), u32::MAX);
    let hash = board.state_hash();
    assert!(!board.apply(Op::page_add(OpId::of(0, 1), u32::MAX)));
    assert_eq!(board.state_hash(), hash);
}
