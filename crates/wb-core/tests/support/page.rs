//! Page geometry helpers.

use wb_core::{Aabb, WorldPos};

/// Boxes shifted down the page stack.
pub trait AabbExt {
    /// The same box `by` world units lower.
    fn moved_down(self, by: i64) -> Self;
}

impl AabbExt for Aabb {
    fn moved_down(self, by: i64) -> Self {
        Aabb::new(
            WorldPos::new(self.min.x, self.min.y + by),
            WorldPos::new(self.max.x, self.max.y + by),
        )
    }
}
