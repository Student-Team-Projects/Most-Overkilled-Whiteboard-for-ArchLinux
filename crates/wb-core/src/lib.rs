//! Geometry and board state for the whiteboard.

mod board;
mod types;

pub use board::{Board, TombstoneBoard};

pub use types::{
    Aabb, BoardId, EncodedPoints, Lamport, Op, OpId, Outgoing, PeerId, Rgba, Stamp, Stroke,
    StrokeId, StrokePoints, Time, WorldPos,
};
