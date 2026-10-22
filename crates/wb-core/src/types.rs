#[cfg(test)]
#[path = "types_test.rs"]
mod test;

use serde::{Deserialize, Serialize};

/// Random peer identifier for one run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PeerId(
    /// Identifier value.
    pub u64,
);

/// Board identifier containing eight random bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BoardId(
    /// Identifier bytes.
    pub [u8; 8],
);

/// Logical timestamp used for draw order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Lamport(
    /// Logical counter value.
    pub u64,
);

/// Operation identifier composed of its peer and sequence number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OpId {
    /// Peer that created the operation.
    pub peer: PeerId,
    /// Operation sequence number within the peer's run.
    pub seq: u64,
}

/// Identifier of a stroke's Add operation.
pub type StrokeId = OpId;

/// World coordinates, with one unit equal to 1/16 pixel at zoom 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldPos {
    /// Horizontal coordinate in world units.
    pub x: i64,
    /// Vertical coordinate in world units.
    pub y: i64,
}

/// Axis-aligned bounding box in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Aabb {
    /// Minimum coordinates of the bounds.
    pub min: WorldPos,
    /// Maximum coordinates of the bounds.
    pub max: WorldPos,
}

/// Opaque codec bytes containing an i64 origin and packed deltas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncodedPoints(
    /// Encoded point bytes.
    pub Vec<u8>,
);

/// Decoded stroke points represented by an origin and offsets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrokePoints {
    /// Origin in world coordinates.
    pub origin: WorldPos,
    /// Horizontal and vertical offsets from the origin in world units.
    pub offsets: Vec<[i32; 2]>,
}

/// Colour with 8-bit red, green, blue, and alpha channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rgba(
    /// Channels in red, green, blue, alpha order.
    pub [u8; 4],
);

/// Stroke data stored by an Add operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stroke {
    /// Identifier of the Add operation.
    pub id: StrokeId,
    /// Logical timestamp used for draw order.
    pub lamport: Lamport,
    /// Stroke colour.
    pub color: Rgba,
    /// Stroke width in world units.
    pub width: u32,
    /// Bounding box in world coordinates.
    pub bbox: Aabb,
    /// Encoded stroke points.
    pub points: EncodedPoints,
}

/// Board operation, extensible with additional variants.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Op {
    /// Add a stroke to the board.
    Add(Stroke),
    /// Erase a stroke by its Add operation's identifier.
    Erase {
        /// Identifier of this erase operation.
        id: OpId,
        /// Identifier of the stroke to erase.
        target: StrokeId,
    },
}

/// Injected milliseconds supplied by a driver, never read from the OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Time(
    /// Millisecond value.
    pub u64,
);

/// Sans-IO effect carrying a message for one peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Outgoing<M> {
    /// Destination peer.
    pub to: PeerId,
    /// Message to send.
    pub msg: M,
}
