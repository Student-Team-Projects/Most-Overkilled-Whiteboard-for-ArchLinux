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

impl OpId {
    /// Builds the id of operation number `seq` authored by `peer`.
    #[must_use]
    pub const fn new(peer: PeerId, seq: u64) -> Self {
        Self { peer, seq }
    }
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

impl WorldPos {
    /// Builds the position at `x`, `y` world units.
    #[must_use]
    pub const fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }
}

/// Axis-aligned bounding box in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Aabb {
    /// Minimum coordinates of the bounds.
    pub min: WorldPos,
    /// Maximum coordinates of the bounds.
    pub max: WorldPos,
}

impl Aabb {
    /// Builds the box spanning `min` to `max`; the caller keeps `min <= max` on both axes.
    #[must_use]
    pub const fn new(min: WorldPos, max: WorldPos) -> Self {
        Self { min, max }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postcard_round_trip() -> Result<(), postcard::Error> {
        let id = OpId::new(PeerId(42), 7);
        let add = Op::Add(Stroke {
            id,
            lamport: Lamport(19),
            color: Rgba([12, 34, 56, 200]),
            width: 32,
            bbox: Aabb::new(WorldPos::new(-160, -32), WorldPos::new(320, 640)),
            points: EncodedPoints(vec![0, 127, 128, 255]),
        });
        let erase = Op::Erase {
            id: OpId::new(PeerId(43), 8),
            target: id,
        };
        let original = (
            BoardId([0, 1, 2, 3, 4, 5, 254, 255]),
            add,
            erase.clone(),
            Outgoing {
                to: PeerId(44),
                msg: erase,
            },
        );
        let bytes = postcard::to_allocvec(&original)?;
        let decoded: (BoardId, Op, Op, Outgoing<Op>) = postcard::from_bytes(&bytes)?;
        assert_eq!(decoded, original);
        Ok(())
    }
}
