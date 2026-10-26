//! Lookup of strokes by bounding box, and `LinearIndex`, the simplest index.

use crate::{Aabb, StrokeId};

/// Answers "which strokes lie in this area?" by bounding box, for the eraser and the renderer.
///
/// An index holds at most one box per stroke id. Boxes are closed: a box includes its edges, so
/// two boxes that share only an edge or a corner intersect. Every box passed in must have
/// `min <= max` on both axes. Any `i64` coordinate works, up to and including `i64::MIN` and
/// `i64::MAX`: the world has no fixed bounds. No operation fails or panics.
///
/// Results are by box only; whether a stroke's actual line touches the area is the caller's
/// check.
///
/// See also: `docs/contracts/spatial-index.md`.
pub trait SpatialIndex {
    /// Stores `bbox` as the bounds of `id`. If `id` is already present, `bbox` replaces its old
    /// bounds, so from then on `id` is found only at `bbox`.
    fn insert(&mut self, id: StrokeId, bbox: Aabb);

    /// Forgets `id`. Returns `true` if it was present, and `false`, changing nothing, if not.
    fn remove(&mut self, id: StrokeId) -> bool;

    /// Appends to `out` exactly the present ids whose box intersects `area`: none is missed, no
    /// other id is added, and each appears once, in no particular order. Ids already in `out`
    /// stay.
    fn query(&self, area: Aabb, out: &mut Vec<StrokeId>);
}

/// The reference index: every entry in a `Vec`, every query a scan of all of them. Faster
/// indexes are tested against it.
#[derive(Debug, Default)]
pub struct LinearIndex {
    entries: Vec<(StrokeId, Aabb)>,
}

impl LinearIndex {
    /// Builds an empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl SpatialIndex for LinearIndex {
    fn insert(&mut self, id: StrokeId, bbox: Aabb) {
        match self.entries.iter_mut().find(|(present, _)| *present == id) {
            Some(entry) => entry.1 = bbox,
            None => self.entries.push((id, bbox)),
        }
    }

    fn remove(&mut self, id: StrokeId) -> bool {
        let Some(index) = self.entries.iter().position(|(present, _)| *present == id) else {
            return false;
        };
        self.entries.swap_remove(index);
        true
    }

    fn query(&self, area: Aabb, out: &mut Vec<StrokeId>) {
        let hits = self
            .entries
            .iter()
            .filter(|(_, bbox)| intersects(*bbox, area));
        out.extend(hits.map(|(id, _)| *id));
    }
}

/// Whether closed boxes `a` and `b` overlap on both axes; sharing an edge or a corner counts.
fn intersects(a: Aabb, b: Aabb) -> bool {
    a.min.x <= b.max.x && b.min.x <= a.max.x && a.min.y <= b.max.y && b.min.y <= a.max.y
}
