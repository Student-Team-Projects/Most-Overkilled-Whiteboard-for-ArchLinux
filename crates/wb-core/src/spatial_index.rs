//! "Which strokes are in this area?", asked by the eraser and the renderer
//! (`docs/contracts/spatial-index.md`).

mod linear;

#[cfg(test)]
mod conformance;
#[cfg(test)]
mod testing;

pub use linear::LinearIndex;

use crate::{Aabb, StrokeId};

/// Finds stroke ids by bounding box.
///
/// Bounds are closed: boxes that share only an edge or a corner intersect. Callers pass boxes
/// with `min <= max` on both axes, which is what an `Aabb`'s min and max mean
/// (`docs/contracts/README.md`).
pub trait SpatialIndex {
    /// Inserts `id` with `bbox`, or replaces the bounds of `id` if it is present.
    ///
    /// Contract: `spatial-index.md` → `insert`, guarantee 2.
    fn insert(&mut self, id: StrokeId, bbox: Aabb);

    /// Removes `id` and returns whether it was present.
    ///
    /// Contract: `spatial-index.md` → `remove`.
    fn remove(&mut self, id: StrokeId) -> bool;

    /// Appends to `out`, once each and in no particular order, every present id whose bounds
    /// intersect `area`.
    ///
    /// Contract: `spatial-index.md` → `query`, guarantees 1 and 3.
    fn query(&self, area: Aabb, out: &mut Vec<StrokeId>);
}
