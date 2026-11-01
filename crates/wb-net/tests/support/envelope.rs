//! Test packets and their generators.

use proptest::prelude::*;
use wb_core::{Aabb, BoardId, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};
use wb_net::{BoardMsg, Body, Packet};

use super::config::MAX_PAYLOAD;

/// A packet whose discovery and sync messages are plain bytes: the envelope never looks inside.
pub type TestPacket = Packet<Vec<u8>, Vec<u8>>;

fn op_id() -> impl Strategy<Value = OpId> {
    (any::<u64>(), any::<u64>()).prop_map(|(peer, seq)| OpId::new(PeerId(peer), seq))
}

fn op() -> impl Strategy<Value = Op> {
    let pos = || (any::<i64>(), any::<i64>()).prop_map(|(x, y)| WorldPos::new(x, y));
    let add = (
        op_id(),
        any::<u64>(),
        any::<[u8; 4]>(),
        any::<u32>(),
        pos(),
        pos(),
        prop::collection::vec(any::<u8>(), 0..=MAX_PAYLOAD),
    )
        .prop_map(|(id, lamport, color, width, min, max, points)| {
            Op::Add(Stroke {
                id,
                lamport: Lamport(lamport),
                color: Rgba(color),
                width,
                bbox: Aabb::new(min, max),
                points: EncodedPoints(points),
            })
        });
    let erase = (op_id(), op_id()).prop_map(|(id, target)| Op::Erase { id, target });
    prop_oneof![add, erase]
}

/// Any packet of every shape: discovery, a board op, or board sync traffic.
pub fn packet() -> impl Strategy<Value = TestPacket> {
    let bytes = || prop::collection::vec(any::<u8>(), 0..=MAX_PAYLOAD);
    let msg = prop_oneof![
        op().prop_map(BoardMsg::Op),
        bytes().prop_map(BoardMsg::Sync)
    ];
    let board = (any::<[u8; 8]>(), msg).prop_map(|(id, msg)| Body::Board {
        board: BoardId(id),
        msg,
    });
    prop_oneof![bytes().prop_map(Body::Discovery), board].prop_map(Packet::new)
}
