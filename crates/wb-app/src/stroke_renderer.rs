//! The `StrokeRenderer` trait and its renderers.

mod naive;

pub use naive::Naive;

use egui::{Color32, Painter, Pos2, Stroke};
use wb_core::{Aabb, Board, Rgba, SpatialIndex, WorldPos};

use crate::camera::Camera;
use crate::stroke_cache::StrokeCache;

/// Puts the visible part of a board on screen.
///
/// A frame shows exactly the visible strokes that reach into the viewport, bottom first, as
/// `camera` places them. Points come only through the stroke cache (ADR 0018).
///
/// See also: `docs/contracts/stroke-renderer.md`.
pub trait StrokeRenderer {
    /// Marks whatever the renderer kept for `area` as stale, after an add or an erase there. The
    /// next `paint` shows every change inside `area`.
    fn invalidate(&mut self, area: Aabb);

    /// Paints the strokes of `board` that reach into the viewport: `painter`'s clip rectangle,
    /// centred on the camera's screen centre. Strokes whose points do not decode are skipped.
    fn paint(
        &mut self,
        painter: &Painter,
        camera: &impl Camera,
        board: &impl Board,
        index: &impl SpatialIndex,
        cache: &mut impl StrokeCache,
    );
}

/// Paints `points` as a line `width_in_world_units` wide: a segment between each pair of
/// neighbours and a disc on every point, so ends and joins are round and a lone point is a dot.
pub fn paint_round_line(
    painter: &Painter,
    camera: &impl Camera,
    points: impl Iterator<Item = WorldPos>,
    width_in_world_units: u32,
    color: Rgba,
) {
    let width = camera.world_length_to_screen(width_in_world_units);
    let color = to_color32(color);
    let on_screen: Vec<Pos2> = points.map(|point| camera.world_to_screen(point)).collect();
    for segment in on_screen.windows(2) {
        painter.line_segment([segment[0], segment[1]], Stroke::new(width, color));
    }
    for &point in &on_screen {
        painter.circle_filled(point, width / 2.0, color);
    }
}

/// The egui colour of a stroke colour.
#[must_use]
pub fn to_color32(Rgba([r, g, b, a]): Rgba) -> Color32 {
    Color32::from_rgba_unmultiplied(r, g, b, a)
}
