//! Geometry and board state for the whiteboard.

mod spatial_index;
mod types;

pub use spatial_index::{LinearIndex, SpatialIndex};

pub use types::{
    Aabb, BoardId, EncodedPoints, Lamport, Op, OpId, Outgoing, PeerId, Rgba, Stroke, StrokeId,
    StrokePoints, Time, WorldPos,
};
