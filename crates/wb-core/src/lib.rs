//! Geometry and board state for the whiteboard.

mod codec;
mod types;

pub use codec::{CodecError, Raw, StrokeCodec};

pub use types::{
    Aabb, BoardId, EncodedPoints, Lamport, Op, OpId, Outgoing, PeerId, Rgba, Stroke, StrokeId,
    StrokePoints, Time, WorldPos,
};
