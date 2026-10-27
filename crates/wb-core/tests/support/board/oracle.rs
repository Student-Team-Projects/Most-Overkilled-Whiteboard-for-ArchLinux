//! What a board built from some ops must hold, computed from the ops alone.

use std::collections::BTreeSet;

use wb_core::{Lamport, Op, OpId, Stroke};

use super::OpExt;
use crate::support::config::CANONICAL_ERASER;

fn strokes(ops: &[Op]) -> impl Iterator<Item = &Stroke> {
    ops.iter().filter_map(|op| match op {
        Op::Add(stroke) => Some(stroke),
        Op::Erase { .. } => None,
    })
}

fn erase_targets(ops: &[Op]) -> BTreeSet<OpId> {
    ops.iter()
        .filter_map(|op| match op {
            Op::Erase { target, .. } => Some(*target),
            Op::Add(_) => None,
        })
        .collect()
}

/// Unerased strokes sorted by `(lamport, peer, seq)`.
pub fn picture(ops: &[Op]) -> Vec<Stroke> {
    let erased = erase_targets(ops);
    let mut picture: Vec<Stroke> = strokes(ops)
        .filter(|stroke| !erased.contains(&stroke.id))
        .cloned()
        .collect();
    picture.sort_by_key(|stroke| (stroke.lamport, stroke.id));
    picture
}

/// One op per item of the canonical state: the first Add of each stroke id, then one erase of
/// each erased target by `CANONICAL_ERASER`, with op ids the original erases never had.
pub fn canonical(ops: &[Op]) -> Vec<Op> {
    let mut added = BTreeSet::new();
    let adds = strokes(ops).filter(|stroke| added.insert(stroke.id));
    let adds = adds.map(|stroke| Op::Add(stroke.clone()));
    let erases = (0..)
        .zip(erase_targets(ops))
        .map(|(seq, target)| Op::erase(OpId::new(CANONICAL_ERASER, seq), target));
    adds.chain(erases).collect()
}

/// Every id the ops use: op ids and erase targets.
pub fn ids(ops: &[Op]) -> BTreeSet<OpId> {
    ops.iter().map(Op::id).chain(erase_targets(ops)).collect()
}

/// The highest Lamport of any Add, if there is one.
pub fn max_lamport(ops: &[Op]) -> Option<Lamport> {
    strokes(ops).map(|stroke| stroke.lamport).max()
}
