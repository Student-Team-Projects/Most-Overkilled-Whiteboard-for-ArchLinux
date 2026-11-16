//! A board with its index and cache, painted off screen, for the `StrokeRenderer` tests.

use egui::{Color32, Context, Pos2, RawInput, Rect, Shape};
use wb_app::camera::AffineCamera;
use wb_app::stroke_renderer::StrokeRenderer;
use wb_app::{DecodeAll, StrokeCache};
use wb_core::{
    Aabb, Board, LinearIndex, Op, Raw, Rgba, SpatialIndex, StrokeCodec, StrokeId, TombstoneBoard,
    WorldPos,
};

use super::config::{AUTHOR, BUDGET, OPENING_ZOOM, VIEWPORT, WIDTH};

/// What a renderer reads, kept in step the way the app keeps it.
pub struct Scene {
    board: TombstoneBoard,
    index: LinearIndex,
    cache: DecodeAll<Raw>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            board: TombstoneBoard::new(),
            index: LinearIndex::new(),
            cache: DecodeAll::new(Raw),
        }
    }

    /// Adds a one-piece stroke through `points` and returns its id and bounds; `None` if the
    /// points need more than one piece.
    pub fn add(&mut self, points: &[WorldPos], color: Rgba) -> Option<(StrokeId, Aabb)> {
        let [piece] = <[_; 1]>::try_from(Raw.encode(points, BUDGET).ok()?).ok()?;
        let bbox = Aabb::covering(&Raw.decode(&piece).ok()?, WIDTH);
        let stamp = self.board.stamp(AUTHOR);
        self.board.apply(Op::add(stamp, color, WIDTH, piece, bbox));
        self.index.insert(stamp.id, bbox);
        Some((stamp.id, bbox))
    }

    pub fn erase(&mut self, target: StrokeId) {
        let stamp = self.board.stamp(AUTHOR);
        self.board.apply(Op::erase(stamp, target));
        self.index.remove(target);
        self.cache.forget(target);
    }

    /// The colours of the dots `renderer` paints in one frame of a `VIEWPORT`-sized screen whose
    /// centre shows the world origin, in paint order. Every painted point has one dot.
    pub fn painted_dots(&mut self, renderer: &mut impl StrokeRenderer) -> Vec<Color32> {
        let screen = Rect::from_min_size(Pos2::ZERO, VIEWPORT);
        let camera = AffineCamera::new(WorldPos::new(0, 0), screen.center(), OPENING_ZOOM);
        let input = RawInput {
            screen_rect: Some(screen),
            ..RawInput::default()
        };
        let mut output = Context::default().run_ui(input, |ui| {
            renderer.paint(
                ui.painter(),
                &camera,
                &self.board,
                &self.index,
                &mut self.cache,
            );
        });
        output.textures_delta.clear();
        output
            .shapes
            .into_iter()
            .filter_map(|clipped| match clipped.shape {
                Shape::Circle(dot) => Some(dot.fill),
                _ => None,
            })
            .collect()
    }
}
