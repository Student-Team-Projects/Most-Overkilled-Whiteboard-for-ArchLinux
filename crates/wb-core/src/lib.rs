//! Geometry and board state for the whiteboard.

mod board;
mod types;

pub use board::{Board, ItemHasher, TombstoneBoard, Xxh3};

pub use types::{
    Aabb, BoardId, EncodedPoints, Lamport, Op, OpId, Outgoing, PeerId, Rgba, Stroke, StrokeId,
    StrokePoints, Time, WorldPos,
};
