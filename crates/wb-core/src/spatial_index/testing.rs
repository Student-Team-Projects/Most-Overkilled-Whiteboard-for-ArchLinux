//! Test API for spatial indexes: boxes, generated steps, and the model every index must agree
//! with.

use std::collections::BTreeMap;

use crate::{Aabb, OpId, PeerId, StrokeId, WorldPos};

/// Stroke id number `n`; ids sort by `n`.
pub fn id(n: u64) -> StrokeId {
    OpId {
        peer: PeerId(0),
        seq: n,
    }
}

/// The closed box from `(x0, y0)` to `(x1, y1)`.
pub fn aabb(x0: i64, y0: i64, x1: i64, y1: i64) -> Aabb {
    Aabb {
        min: WorldPos { x: x0, y: y0 },
        max: WorldPos { x: x1, y: y1 },
    }
}

/// One step of a session against an index.
#[derive(Debug, Clone)]
pub enum Step {
    Insert(StrokeId, Aabb),
    Remove(StrokeId),
    Query(Aabb),
}

/// The reference every index must agree with: a map from id to bounds, where a box hits an area
/// when their ranges overlap on both axes. `LinearIndex` cannot be its own oracle, so it and the
/// later `QuadTree` both answer to this model on the same sessions, which also compares them.
#[derive(Debug, Default)]
pub struct Model(BTreeMap<StrokeId, Aabb>);

impl Model {
    /// Inserts `id`, or replaces its bounds.
    pub fn insert(&mut self, id: StrokeId, bbox: Aabb) {
        self.0.insert(id, bbox);
    }

    /// Removes `id` and returns whether it was present.
    pub fn remove(&mut self, id: StrokeId) -> bool {
        self.0.remove(&id).is_some()
    }

    /// Ids whose bounds hit `area`, sorted.
    pub fn query(&self, area: Aabb) -> Vec<StrokeId> {
        let hit = |bbox: &Aabb| {
            overlap((bbox.min.x, bbox.max.x), (area.min.x, area.max.x))
                && overlap((bbox.min.y, bbox.max.y), (area.min.y, area.max.y))
        };
        self.0
            .iter()
            .filter(|(_, bbox)| hit(bbox))
            .map(|(id, _)| *id)
            .collect()
    }
}

/// Whether the closed ranges `[a.0, a.1]` and `[b.0, b.1]` share a point.
fn overlap(a: (i64, i64), b: (i64, i64)) -> bool {
    a.0.max(b.0) <= a.1.min(b.1)
}

/// Generated boxes and sessions.
pub mod strategy {
    use proptest::prelude::*;

    use super::{Step, aabb, id};
    use crate::Aabb;

    /// Ids come from a pool large enough that a `QuadTree` must split its nodes, yet small enough
    /// that inserts often replace and removes often hit.
    const IDS: u64 = 64;
    /// Most steps in one session.
    const STEPS: usize = 256;

    /// Mostly a small grid, where boxes overlap and touch often; sometimes any `i64`, including
    /// exactly the extremes.
    fn coordinate() -> impl Strategy<Value = i64> {
        prop_oneof![
            8 => -8..=8i64,
            1 => any::<i64>(),
            1 => Just(i64::MIN),
            1 => Just(i64::MAX),
        ]
    }

    /// A closed box with `min <= max` on both axes.
    pub fn bbox() -> impl Strategy<Value = Aabb> {
        [coordinate(), coordinate(), coordinate(), coordinate()]
            .prop_map(|[x0, x1, y0, y1]| aabb(x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1)))
    }

    /// A random session: inserts outweigh removes, so the index grows past what one node holds.
    pub fn session() -> impl Strategy<Value = Vec<Step>> {
        let step = prop_oneof![
            3 => (0..IDS, bbox()).prop_map(|(n, bbox)| Step::Insert(id(n), bbox)),
            1 => (0..IDS).prop_map(|n| Step::Remove(id(n))),
            2 => bbox().prop_map(Step::Query),
        ];
        prop::collection::vec(step, 0..STEPS)
    }
}
