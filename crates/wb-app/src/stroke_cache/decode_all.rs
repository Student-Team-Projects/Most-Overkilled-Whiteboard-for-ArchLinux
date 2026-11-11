//! The simplest cache: decode each stroke once and keep it.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use wb_core::{Aabb, CodecError, Stroke, StrokeCodec, StrokeId, StrokePoints};

use super::StrokeCache;

/// Decodes a stroke on its first `get` and keeps the points until `forget`; `set_resident` drops
/// nothing.
#[derive(Debug)]
pub struct DecodeAll<C> {
    codec: C,
    points: HashMap<StrokeId, StrokePoints>,
}

impl<C: StrokeCodec> DecodeAll<C> {
    /// An empty cache that decodes with `codec`.
    #[must_use]
    pub fn new(codec: C) -> Self {
        Self {
            codec,
            points: HashMap::new(),
        }
    }
}

impl<C: StrokeCodec> StrokeCache for DecodeAll<C> {
    fn get(&mut self, stroke: &Stroke) -> Result<&StrokePoints, CodecError> {
        match self.points.entry(stroke.id) {
            Entry::Occupied(cached) => Ok(cached.into_mut()),
            Entry::Vacant(missing) => Ok(missing.insert(self.codec.decode(&stroke.points)?)),
        }
    }

    fn set_resident(&mut self, _area: Aabb) {}

    fn forget(&mut self, id: StrokeId) {
        self.points.remove(&id);
    }
}
