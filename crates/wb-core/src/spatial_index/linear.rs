//! The simplest index: a linear scan over every stored box.

use super::SpatialIndex;
use crate::{Aabb, Intersects, StrokeId};

/// Keeps every box in a `Vec` and answers each query by scanning all of them.
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
            .filter(|(_, bbox)| bbox.intersects(&area));
        out.extend(hits.map(|(id, _)| *id));
    }
}
