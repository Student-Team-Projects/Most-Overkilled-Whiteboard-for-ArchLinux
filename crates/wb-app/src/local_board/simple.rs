//! The local board built from every layer-0 part: tombstones, a linear index, `Raw`, decode-all
//! and the naive renderer.

use egui::Painter;
use wb_core::{
    Aabb, Board, CodecError, LinearIndex, Op, PeerId, Raw, Rgba, SpatialIndex, StrokeCodec,
    StrokeId, TombstoneBoard, WorldPos,
};

use super::LocalBoard;
use crate::camera::Camera;
use crate::config::PIECE_BUDGET_BYTES;
use crate::eraser_utils::touches;
use crate::stroke_cache::{DecodeAll, StrokeCache};
use crate::stroke_renderer::{Naive, StrokeRenderer};

/// The board replica with its spatial index, stroke cache and renderer. Every change goes
/// through here, so the four never disagree: an added stroke is indexed and shown, an erased one
/// leaves the index and the cache.
#[derive(Debug)]
pub struct SimpleLocalBoard {
    me: PeerId,
    board: TombstoneBoard,
    index: LinearIndex,
    codec: Raw,
    cache: DecodeAll<Raw>,
    renderer: Naive,
}

impl SimpleLocalBoard {
    /// An empty board whose local ops are authored by `me`.
    #[must_use]
    pub fn new(me: PeerId) -> Self {
        Self {
            me,
            board: TombstoneBoard::new(),
            index: LinearIndex::new(),
            codec: Raw,
            cache: DecodeAll::new(Raw),
            renderer: Naive,
        }
    }
}

impl LocalBoard for SimpleLocalBoard {
    type Replica = TombstoneBoard;

    fn board(&self) -> &Self::Replica {
        &self.board
    }

    fn add_stroke(
        &mut self,
        points: &[WorldPos],
        color: Rgba,
        width: u32,
    ) -> Result<(), CodecError> {
        if points.is_empty() {
            return Ok(());
        }
        let pieces = self
            .codec
            .encode(&self.codec.simplify(points), PIECE_BUDGET_BYTES)?;
        let bounds = pieces
            .iter()
            .map(|piece| Ok(Aabb::covering(&self.codec.decode(piece)?, width)))
            .collect::<Result<Vec<_>, CodecError>>()?;
        for (piece, bbox) in pieces.into_iter().zip(bounds) {
            let stamp = self.board.stamp(self.me);
            self.board.apply(Op::add(stamp, color, width, piece, bbox));
            self.index.insert(stamp.id, bbox);
            self.renderer.invalidate(bbox);
        }
        Ok(())
    }

    fn erase_touched(&mut self, centre: WorldPos, reach: i64) {
        let around_centre = Aabb::new(
            WorldPos::new(
                centre.x.saturating_sub(reach),
                centre.y.saturating_sub(reach),
            ),
            WorldPos::new(
                centre.x.saturating_add(reach),
                centre.y.saturating_add(reach),
            ),
        );
        let mut candidates = Vec::new();
        self.index.query(around_centre, &mut candidates);
        let mut touched: Vec<(StrokeId, Aabb)> = Vec::new();
        for stroke in self.board.visible() {
            if candidates.contains(&stroke.id)
                && let Ok(points) = self.cache.get(stroke)
                && touches(points, stroke.width, centre, reach)
            {
                touched.push((stroke.id, stroke.bbox));
            }
        }
        for (target, bbox) in touched {
            let stamp = self.board.stamp(self.me);
            self.board.apply(Op::erase(stamp, target));
            self.index.remove(target);
            self.cache.forget(target);
            self.renderer.invalidate(bbox);
        }
    }

    fn paint(&mut self, painter: &Painter, camera: &impl Camera) {
        self.renderer
            .paint(painter, camera, &self.board, &self.index, &mut self.cache);
    }
}
