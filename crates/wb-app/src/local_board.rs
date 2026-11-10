//! The `LocalBoard` trait and its boards.

mod simple;

pub use simple::SimpleLocalBoard;

use egui::Painter;
use wb_core::{Board, CodecError, Rgba, WorldPos};

use crate::camera::Camera;

/// This peer's board replica plus everything kept in step with it: whatever finds, caches and
/// paints strokes. Every change goes through here, so the parts never disagree: an added stroke
/// is found and shown, an erased one is gone from all of them.
pub trait LocalBoard {
    /// The replica the strokes live in.
    type Replica: Board;

    /// The board replica.
    fn board(&self) -> &Self::Replica;

    /// Adds a stroke through `points` in `color`, `width` world units wide. A stroke too big for
    /// one piece becomes several strokes, each starting where the one before ended (ADR 0020).
    /// No points add nothing.
    ///
    /// # Errors
    ///
    /// The codec's error if the points cannot be encoded; nothing is added then.
    fn add_stroke(
        &mut self,
        points: &[WorldPos],
        color: Rgba,
        width: u32,
    ) -> Result<(), CodecError>;

    /// Erases, whole, every visible stroke that comes within `reach` world units of `centre`
    /// (ADR 0002).
    fn erase_touched(&mut self, centre: WorldPos, reach: i64);

    /// Paints the board into `painter` as `camera` shows it.
    fn paint(&mut self, painter: &Painter, camera: &impl Camera);
}
