//! The `StrokeCache` trait and its caches.

mod decode_all;

pub use decode_all::DecodeAll;

use wb_core::{Aabb, CodecError, Stroke, StrokeId, StrokePoints};

/// Keeps decoded stroke points, so the renderer and the eraser do not decode every frame. A cache
/// changes only how fast points come back, never which points.
///
/// Strokes are told apart by id: one id must always come with the same points, as on a board.
pub trait StrokeCache {
    /// Returns `stroke`'s points as the cache's codec decodes them, from the cache or decoded now.
    ///
    /// # Errors
    ///
    /// The codec's error if `stroke`'s points do not decode.
    fn get(&mut self, stroke: &Stroke) -> Result<&StrokePoints, CodecError>;

    /// Sets the resident area: the world area on screen plus a margin. The cache may drop decoded
    /// points of strokes outside it to save memory; a later `get` decodes them again.
    fn set_resident(&mut self, area: Aabb);

    /// Drops whatever the cache holds for `id`, an erased stroke.
    fn forget(&mut self, id: StrokeId);
}
