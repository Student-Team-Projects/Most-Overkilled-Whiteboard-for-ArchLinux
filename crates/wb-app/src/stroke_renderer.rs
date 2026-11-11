//! The `StrokeRenderer` trait and its renderers.

mod naive;

pub use naive::Naive;

use egui::Painter;
use wb_core::{Aabb, Board, SpatialIndex};

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

    /// Paints the strokes of `board` that reach into the viewport, `painter`'s clip rectangle.
    /// Strokes whose points do not decode are skipped.
    fn paint(
        &mut self,
        painter: &Painter,
        camera: &impl Camera,
        board: &impl Board,
        index: &impl SpatialIndex,
        cache: &mut impl StrokeCache,
    );
}
