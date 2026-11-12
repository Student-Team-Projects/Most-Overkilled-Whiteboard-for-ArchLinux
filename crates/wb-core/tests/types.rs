//! Wire round trip of the shared domain types.

use wb_core::{
    Aabb, BoardId, EncodedPoints, Lamport, Op, OpId, Outgoing, PeerId, Rgba, Stroke, WorldPos,
};

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
