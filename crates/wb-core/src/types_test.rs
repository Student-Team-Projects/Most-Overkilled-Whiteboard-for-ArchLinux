use super::*;

#[test]
fn postcard_round_trip() -> Result<(), postcard::Error> {
    let id = OpId {
        peer: PeerId(42),
        seq: 7,
    };
    let add = Op::Add(Stroke {
        id,
        lamport: Lamport(19),
        color: Rgba([12, 34, 56, 200]),
        width: 32,
        bbox: Aabb {
            min: WorldPos { x: -160, y: -32 },
            max: WorldPos { x: 320, y: 640 },
        },
        points: EncodedPoints(vec![0, 127, 128, 255]),
    });
    let erase = Op::Erase {
        id: OpId {
            peer: PeerId(43),
            seq: 8,
        },
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
