//! The simplest renderer: every visible stroke, every frame.

use egui::{Painter, Pos2, Stroke};
use wb_core::{Aabb, Board, Intersects, Rgba, SpatialIndex, WorldPos};

use super::{StrokeRenderer, to_color32};
use crate::camera::Camera;
use crate::stroke_cache::StrokeCache;

/// Walks `board.visible()` each frame and paints every stroke whose box meets the viewport. Keeps
/// nothing between frames, so `invalidate` has nothing to do. The oracle for faster renderers.
#[derive(Debug, Default)]
pub struct Naive;

impl Naive {
    /// Paints `points` as a line `width_in_world_units` wide: a segment between each pair of
    /// neighbours and a disc on every point, so ends and joins are round and a lone point is a
    /// dot.
    pub fn paint_round_line(
        painter: &Painter,
        camera: &impl Camera,
        points: &[WorldPos],
        width_in_world_units: u32,
        color: Rgba,
    ) {
        let width = camera.world_length_to_screen(width_in_world_units);
        let color = to_color32(color);
        let on_screen: Vec<Pos2> = points
            .iter()
            .map(|&point| camera.world_to_screen(point))
            .collect();
        for segment in on_screen.windows(2) {
            painter.line_segment([segment[0], segment[1]], Stroke::new(width, color));
        }
        for &point in &on_screen {
            painter.circle_filled(point, width / 2.0, color);
        }
    }
}

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
                Self::paint_round_line(
                    painter,
                    camera,
                    &Vec::from(points),
                    stroke.width,
                    stroke.color,
                );
            }
        }
    }
}
