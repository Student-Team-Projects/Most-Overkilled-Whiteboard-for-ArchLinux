//! Layer-0 index: a `Vec` and a linear scan (`spatial-index.md` → Impls).

#[cfg(test)]
#[path = "linear_test.rs"]
mod test;

use super::SpatialIndex;
use crate::{Aabb, StrokeId};

/// The oracle index: every entry in a `Vec`, every query a scan of all of them.
#[derive(Debug, Default)]
pub struct LinearIndex {
    entries: Vec<(StrokeId, Aabb)>,
}

impl LinearIndex {
    /// An empty index.
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
