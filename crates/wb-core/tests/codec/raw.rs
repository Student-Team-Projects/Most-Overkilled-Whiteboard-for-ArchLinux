//! `Raw` passes the shared `StrokeCodec` suite; the tests below pin its own layout, split points,
//! minimum budget and strict decoding.

use proptest::prelude::*;
use wb_core::{CodecError, EncodedPoints, Raw, StrokeCodec, StrokePoints, WorldPos};

use crate::conformance::codec::stroke_codec_conformance;
use crate::support::codec::{expand, strategy};
use crate::support::config::{BUDGET, LONG_STROKE, NEAR_ZERO, RAW_MIN_BUDGET, WIDE_OFFSET};

stroke_codec_conformance!(Raw, tolerance = 0.0, min_budget = RAW_MIN_BUDGET);

/// Offsets that overflow `i32` from the origin split at the last point that still fits.
#[test]
fn splits_where_offset_overflows_i32() {
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

/// A one-point stroke fits a budget of exactly its own size; a second point then needs a split
/// that cannot happen, since every piece needs two points to make progress.
#[test]
fn second_point_does_not_fit_a_one_point_budget() {
    let dot = WorldPos::new(-5, 3);
    let size = Raw.encode(&[dot], BUDGET).unwrap()[0].0.len();
    assert!(Raw.encode(&[dot], size).is_ok());
    assert_eq!(
        Raw.encode(&[dot, dot], size),
        Err(CodecError::BudgetTooSmall)
    );
}

/// A length prefix far beyond the bytes present is an `Err`, not a huge allocation or a panic.
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

/// Such a piece would overflow `i64` when expanded; `encode` never writes one.
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

/// Every single-byte change of a piece at the `i64` extremes, where overflow is one byte away,
/// either fails to decode or decodes to offsets that expand without overflowing `i64`.
#[test]
fn decode_never_yields_offsets_that_overflow_i64() {
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

/// An origin anywhere, from 1-byte varints near zero to 10-byte ones near the `i64` extremes.
fn origin() -> impl Strategy<Value = WorldPos> {
    (strategy::coordinate(1), strategy::coordinate(1)).prop_map(|(x, y)| WorldPos::new(x, y))
}

/// Offsets near zero, or wide ones that mostly need 5 bytes yet keep consecutive steps within `i32`.
fn offset() -> impl Strategy<Value = [i32; 2]> {
    let axis = prop_oneof![-NEAR_ZERO..NEAR_ZERO, (1 - WIDE_OFFSET)..WIDE_OFFSET];
    [axis.clone(), axis]
}

proptest! {
    /// A piece fills the budget to the byte: the size `Raw` tracks equals postcard's output,
    /// including length prefixes past 127 points and 10-byte origins.
    #[test]
    fn fills_budget_exactly(
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

/// `RAW_MIN_BUDGET` is exactly the largest two-point piece: 10-byte origins and 5-byte offsets.
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
