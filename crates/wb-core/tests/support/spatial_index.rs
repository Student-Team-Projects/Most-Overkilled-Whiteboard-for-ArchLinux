//! Generated sessions against a `SpatialIndex`, and the model every index must agree with.

use std::collections::BTreeMap;

use proptest::prelude::*;
use wb_core::{Aabb, PeerId, StrokeId, WorldPos};

use super::config::{GRID, ID_POOL, MAX_STEPS};

/// One step of a session against an index.
#[derive(Debug, Clone)]
pub enum Step {
    /// Insert or replace the bounds of a stroke.
    Insert(StrokeId, Aabb),
    /// Remove a stroke, present or not.
    Remove(StrokeId),
    /// Ask for the strokes in an area.
    Query(Aabb),
}

/// The reference every index must agree with: a map from id to bounds, where a box hits an area
/// when their closed ranges share a point on both axes. `LinearIndex` cannot be compared with
/// itself, so it and every faster index answer to this independent model instead.
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

/// Mostly a small grid, where boxes overlap and touch often; sometimes any `i64`, including
/// exactly the extremes.
fn coordinate() -> impl Strategy<Value = i64> {
    prop_oneof![
        8 => -GRID..=GRID,
        1 => any::<i64>(),
        1 => Just(i64::MIN),
        1 => Just(i64::MAX),
    ]
}

/// A closed box with `min <= max` on both axes.
fn bbox() -> impl Strategy<Value = Aabb> {
    [coordinate(), coordinate(), coordinate(), coordinate()].prop_map(|[x0, x1, y0, y1]| {
        Aabb::new(
            WorldPos::new(x0.min(x1), y0.min(y1)),
            WorldPos::new(x0.max(x1), y0.max(y1)),
        )
    })
}

/// A stroke id from the pool of `ID_POOL` ids.
fn stroke_id() -> impl Strategy<Value = StrokeId> {
    (0..ID_POOL).prop_map(|seq| StrokeId::new(PeerId(0), seq))
}

/// A random session: inserts outweigh removes, so the index holds dozens of strokes at once.
pub fn session() -> impl Strategy<Value = Vec<Step>> {
    let step = prop_oneof![
        3 => (stroke_id(), bbox()).prop_map(|(id, bbox)| Step::Insert(id, bbox)),
        1 => stroke_id().prop_map(Step::Remove),
        2 => bbox().prop_map(Step::Query),
    ];
    prop::collection::vec(step, 0..MAX_STEPS)
}
