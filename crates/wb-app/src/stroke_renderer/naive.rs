//! The simplest renderer: every visible stroke, every frame.

use egui::Painter;
use wb_core::{Aabb, Board, Intersects, SpatialIndex};

use super::{StrokeRenderer, paint_round_line};
use crate::camera::Camera;
use crate::eraser_utils::world_points;
use crate::stroke_cache::StrokeCache;

/// Walks `board.visible()` each frame and paints every stroke whose box meets the viewport. Keeps
/// nothing between frames, so `invalidate` has nothing to do. The oracle for faster renderers.
#[derive(Debug, Default)]
pub struct Naive;

impl StrokeRenderer for Naive {
    fn invalidate(&mut self, _area: Aabb) {}

    fn paint(
        &mut self,
        painter: &Painter,
        camera: &impl Camera,
        board: &impl Board,
        _index: &impl SpatialIndex,
        cache: &mut impl StrokeCache,
    ) {
        let visible_area = camera.visible_area(painter.clip_rect().size());
        cache.set_resident(visible_area);
        for stroke in board.visible() {
            if !stroke.bbox.intersects(&visible_area) {
                continue;
            }
            if let Ok(points) = cache.get(stroke) {
                paint_round_line(
                    painter,
                    camera,
                    world_points(points),
                    stroke.width,
                    stroke.color,
                );
            }
        }
    }
}
