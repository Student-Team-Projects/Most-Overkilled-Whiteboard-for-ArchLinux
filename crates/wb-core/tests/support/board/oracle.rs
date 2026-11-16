//! What a board built from some ops must hold, computed from the ops alone.

use std::collections::BTreeSet;

use wb_core::{Lamport, Op, OpId, Stroke};

use super::OpExt;
use crate::support::config::{CANONICAL_AUTHOR, PAGE_PITCH};

fn strokes(ops: &[Op]) -> impl Iterator<Item = &Stroke> {
    ops.iter().filter_map(|op| match op {
        Op::Add(stroke) => Some(stroke),
        Op::Erase { .. } | Op::AddPage { .. } => None,
    })
}

fn erase_targets(ops: &[Op]) -> BTreeSet<OpId> {
    ops.iter()
        .filter_map(|op| match op {
            Op::Erase { target, .. } => Some(*target),
            Op::Add(_) | Op::AddPage { .. } => None,
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

/// How many pages the ops make: at least 1, every add-page count, and one past the page of every
/// stroke's lowest point, where a page owns the gap below it. Generated strokes never go above
/// the first page.
pub fn page_count(ops: &[Op]) -> u32 {
    let asked = ops.iter().filter_map(|op| match op {
        Op::AddPage { count, .. } => Some(*count),
        Op::Add(_) | Op::Erase { .. } => None,
    });
    let held = strokes(ops)
        .map(|stroke| u32::try_from(stroke.bbox.max.y / PAGE_PITCH + 1).unwrap_or(u32::MAX));
    asked.chain(held).fold(1, u32::max)
}

/// One op per item of the canonical state: the first Add of each stroke id, one erase of each
/// erased target, and one add-page op for the page count, all by `CANONICAL_AUTHOR` with op ids
/// the original ops never had.
pub fn canonical(ops: &[Op]) -> Vec<Op> {
    let mut added = BTreeSet::new();
    let adds = strokes(ops).filter(|stroke| added.insert(stroke.id));
    let adds = adds.map(|stroke| Op::Add(stroke.clone()));
    let erases = erase_targets(ops).into_iter().map(Some);
    let page_add = std::iter::once(None);
    let canonical_ops = (0..).zip(erases.chain(page_add)).map(|(seq, item)| {
        let id = OpId::new(CANONICAL_AUTHOR, seq);
        match item {
            Some(target) => Op::erasure(id, target),
            None => Op::page_add(id, page_count(ops)),
        }
    });
    adds.chain(canonical_ops).collect()
}

/// Every id the ops use: op ids and erase targets.
pub fn ids(ops: &[Op]) -> BTreeSet<OpId> {
    ops.iter().map(Op::id).chain(erase_targets(ops)).collect()
}

/// The highest Lamport of any Add, if there is one.
pub fn max_lamport(ops: &[Op]) -> Option<Lamport> {
    strokes(ops).map(|stroke| stroke.lamport).max()
}
