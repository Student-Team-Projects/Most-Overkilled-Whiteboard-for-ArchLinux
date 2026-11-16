//! `Raw`'s own tests: layout, split points, minimum budget and decoding.

use proptest::prelude::*;
use wb_core::{CodecError, EncodedPoints, Raw, StrokeCodec, StrokePoints, WorldPos};

use crate::support::codec::{expand, strategy};
use crate::support::config::{BUDGET, LONG_STROKE, NEAR_ZERO, RAW_MIN_BUDGET, WIDE_OFFSET};

#[test]
fn splits_at_the_last_point_whose_offset_fits_i32() {
    let max = i64::from(i32::MAX);
    let points = [
        WorldPos::new(0, 0),
        WorldPos::new(max, 0),
        WorldPos::new(2 * max, 0),
    ];
    let pieces: Vec<StrokePoints> = Raw
        .encode(&points, BUDGET)
        .unwrap()
        .iter()
        .map(|piece| Raw.decode(piece).unwrap())
        .collect();
    let piece = |origin| StrokePoints {
        origin,
        offsets: vec![[0, 0], [i32::MAX, 0]],
    };
    assert_eq!(pieces, [piece(points[0]), piece(points[1])]);
}

#[test]
fn empty_stroke_is_one_piece_without_points() {
    let pieces = Raw.encode(&[], BUDGET).unwrap();
    let empty = StrokePoints {
        origin: WorldPos::new(0, 0),
        offsets: Vec::new(),
    };
    assert_eq!(
        pieces,
        [EncodedPoints(postcard::to_allocvec(&empty).unwrap())]
    );
}

/// A split piece repeats the shared point, so it needs room for two points.
#[test]
fn two_points_at_a_one_point_budget_is_budget_too_small() {
    let dot = WorldPos::new(-5, 3);
    let size = Raw.encode(&[dot], BUDGET).unwrap()[0].0.len();
    assert!(Raw.encode(&[dot], size).is_ok());
    assert_eq!(
        Raw.encode(&[dot, dot], size),
        Err(CodecError::BudgetTooSmall)
    );
}

#[test]
fn decode_rejects_huge_length_prefix() {
    // Origin (0, 0), then a varint length of 2⁶³ and no offsets.
    let bytes = [
        0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
    ];
    assert_eq!(
        Raw.decode(&EncodedPoints(bytes.to_vec())),
        Err(CodecError::Malformed)
    );
}

#[test]
fn decode_rejects_trailing_bytes() {
    let mut bytes = Raw
        .encode(&[WorldPos::new(1, 2)], BUDGET)
        .unwrap()
        .remove(0)
        .0;
    bytes.push(0);
    assert_eq!(
        Raw.decode(&EncodedPoints(bytes)),
        Err(CodecError::Malformed)
    );
}

#[test]
fn decode_rejects_offsets_that_overflow_i64() {
    for (origin, offset) in [
        (WorldPos::new(i64::MAX, 0), [1, 0]),
        (WorldPos::new(0, i64::MIN), [0, -1]),
    ] {
        let piece = StrokePoints {
            origin,
            offsets: vec![offset],
        };
        let bytes = EncodedPoints(postcard::to_allocvec(&piece).unwrap());
        assert_eq!(Raw.decode(&bytes), Err(CodecError::Malformed));
    }
}

#[test]
fn decode_of_any_byte_change_at_i64_extremes_never_overflows() {
    let (min, max) = (i64::MIN, i64::MAX);
    let extremes = [
        vec![WorldPos::new(max, max)],
        vec![WorldPos::new(min, min)],
        vec![WorldPos::new(max, min), WorldPos::new(max - 1, min + 1)],
    ];
    for raw in extremes {
        for piece in Raw.encode(&raw, BUDGET).unwrap() {
            for position in 0..piece.0.len() {
                for byte in u8::MIN..=u8::MAX {
                    let mut corrupted = piece.0.clone();
                    corrupted[position] = byte;
                    if let Ok(points) = Raw.decode(&EncodedPoints(corrupted)) {
                        assert!(expand(&points).is_some(), "{points:?} overflows i64");
                    }
                }
            }
        }
    }
}

fn origin() -> impl Strategy<Value = WorldPos> {
    (strategy::coordinate(1), strategy::coordinate(1)).prop_map(|(x, y)| WorldPos::new(x, y))
}

fn offset() -> impl Strategy<Value = [i32; 2]> {
    let axis = prop_oneof![-NEAR_ZERO..NEAR_ZERO, (1 - WIDE_OFFSET)..WIDE_OFFSET];
    [axis.clone(), axis]
}

proptest! {
    #[test]
    fn tracked_size_matches_postcard_to_the_byte(
        origin in origin(),
        offsets in prop::collection::vec(offset(), 0..usize::from(LONG_STROKE)),
    ) {
        let whole = StrokePoints {
            origin,
            offsets: [[0, 0]].into_iter().chain(offsets).collect(),
        };
        let bytes = postcard::to_allocvec(&whole).unwrap();
        let points = expand(&whole).unwrap();

        prop_assert_eq!(Raw.encode(&points, bytes.len()), Ok(vec![EncodedPoints(bytes.clone())]));
        let smaller = Raw.encode(&points, bytes.len() - 1);
        prop_assert!(!matches!(smaller, Ok(pieces) if pieces.len() == 1));
    }
}

#[test]
fn min_budget_is_the_largest_two_point_piece() {
    let (min, step) = (i64::MIN, i64::from(i32::MAX));
    let worst = [
        WorldPos::new(min, min),
        WorldPos::new(min + step, min + step),
    ];
    assert_eq!(
        Raw.encode(&worst, RAW_MIN_BUDGET).unwrap()[0].0.len(),
        RAW_MIN_BUDGET
    );
    assert_eq!(
        Raw.encode(&worst, RAW_MIN_BUDGET - 1),
        Err(CodecError::BudgetTooSmall)
    );
}
