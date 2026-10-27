//! Spatial indexes: find strokes by bounding box.

mod linear;

pub use linear::LinearIndex;

use crate::{Aabb, StrokeId};

/// Finds the strokes whose bounding box lies in an area, for the eraser and the renderer. Every
/// box passed in must have `min <= max` on both axes; any `i64` coordinate is allowed.
pub trait SpatialIndex {
    /// Stores `bbox` as the bounds of `id`, replacing any bounds `id` already had.
    fn insert(&mut self, id: StrokeId, bbox: Aabb);

    /// Forgets `id` and returns whether it was present.
    fn remove(&mut self, id: StrokeId) -> bool;

    /// Appends to `out` every stored id whose box intersects `area`, each once and in any order,
    /// keeping what `out` already held. Boxes are closed, so sharing only an edge or a corner
    /// counts as intersecting.
    fn query(&self, area: Aabb, out: &mut Vec<StrokeId>);
}
