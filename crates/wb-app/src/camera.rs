//! The `Camera` trait and its cameras.

mod affine;

pub use affine::AffineCamera;

use egui::{Pos2, Rect, Vec2};
use wb_core::{Aabb, WorldPos};

/// The world ↔ screen transform for pan and zoom. Zoom stays in `[MIN_ZOOM, MAX_ZOOM]`; world
/// points convert to screen points and back within float tolerance.
///
/// See also: `docs/contracts/camera.md`, ADR 0013, ADR 0029.
pub trait Camera {
    /// Screen pixels per world pixel.
    fn zoom(&self) -> f32;

    /// Where `point` appears on screen.
    fn world_to_screen(&self, point: WorldPos) -> Pos2;

    /// The world point shown at `pos`, rounded to whole world units.
    fn screen_to_world(&self, pos: Pos2) -> WorldPos;

    /// How long `length_in_world_units` is on screen.
    fn world_length_to_screen(&self, length_in_world_units: u32) -> f32;

    /// How many world units `length_on_screen` covers, rounded.
    fn screen_length_to_world(&self, length_on_screen: f32) -> i64;

    /// The screen rectangle `area` covers.
    fn world_area_to_screen(&self, area: Aabb) -> Rect {
        Rect::from_min_max(
            self.world_to_screen(area.min),
            self.world_to_screen(area.max),
        )
    }

    /// Moves the view so the world follows the pointer by `screen_delta`.
    fn pan(&mut self, screen_delta: Vec2);

    /// Multiplies the zoom by `factor`, clamped to the allowed range, keeping the world point
    /// under `anchor` there. A non-finite `factor` leaves the zoom as it is.
    fn zoom_at(&mut self, anchor: Pos2, factor: f32);

    /// A world box holding every point shown in a viewport of `viewport_size` centred on the
    /// camera's screen centre.
    fn visible_area(&self, viewport_size: Vec2) -> Aabb;

    /// A camera showing `area` whole and centred in `viewport`, with at least `space_around_px`
    /// screen pixels left free on every side. The zoom stays in range, so in a very small or
    /// very large viewport the area does not fit exactly.
    #[must_use]
    fn fitting(area: Aabb, viewport: Rect, space_around_px: f32) -> Self
    where
        Self: Sized;
}
