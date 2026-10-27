//! Generated sessions against a `SpatialIndex`, and the model every index must agree with.

use std::collections::BTreeMap;

use proptest::prelude::*;
use wb_core::{Aabb, PeerId, StrokeId, WorldPos};

use super::config::{GRID, ID_POOL, MAX_STEPS};

#[derive(Debug, Clone)]
pub enum Step {
    Insert(StrokeId, Aabb),
    Remove(StrokeId),
    Query(Aabb),
}

/// What every index must answer, `LinearIndex` included: it cannot be its own oracle, so this
/// model computes intersection independently.
#[derive(Debug, Default)]
pub struct Model(BTreeMap<StrokeId, Aabb>);

impl Model {
    pub fn insert(&mut self, id: StrokeId, bbox: Aabb) {
        self.0.insert(id, bbox);
    }

    pub fn remove(&mut self, id: StrokeId) -> bool {
        self.0.remove(&id).is_some()
    }

    /// Ids whose box intersects `area`, sorted.
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

fn bbox() -> impl Strategy<Value = Aabb> {
    [coordinate(), coordinate(), coordinate(), coordinate()].prop_map(|[x0, x1, y0, y1]| {
        Aabb::new(
            WorldPos::new(x0.min(x1), y0.min(y1)),
            WorldPos::new(x0.max(x1), y0.max(y1)),
        )
    })
}

fn stroke_id() -> impl Strategy<Value = StrokeId> {
    (0..ID_POOL).prop_map(|seq| StrokeId::new(PeerId(0), seq))
}

/// Inserts outweigh removes, so the index holds dozens of strokes at once.
pub fn session() -> impl Strategy<Value = Vec<Step>> {
    let step = prop_oneof![
        3 => (stroke_id(), bbox()).prop_map(|(id, bbox)| Step::Insert(id, bbox)),
        1 => stroke_id().prop_map(Step::Remove),
        2 => bbox().prop_map(Step::Query),
    ];
    prop::collection::vec(step, 0..MAX_STEPS)
}
