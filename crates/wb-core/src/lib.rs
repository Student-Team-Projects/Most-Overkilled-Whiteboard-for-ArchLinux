//! Geometry and board state for the whiteboard.

mod board;
mod spatial_index;
mod types;

pub use board::{Board, TombstoneBoard};
pub use spatial_index::{LinearIndex, SpatialIndex};

pub use types::{
    Aabb, BoardId, EncodedPoints, Intersects, Lamport, Op, OpId, Outgoing, PeerId, Rgba, Stamp,
    Stroke, StrokeId, StrokePoints, Time, WorldPos,
};
