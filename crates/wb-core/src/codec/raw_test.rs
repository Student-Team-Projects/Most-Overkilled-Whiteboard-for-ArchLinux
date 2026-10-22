//! `Raw` passes the shared `StrokeCodec` suite; the tests below pin its own byte layout.

use proptest::prelude::*;

use super::Raw;
use crate::codec::conformance::stroke_codec_conformance;
use crate::codec::testing::{BUDGET, at};
use crate::codec::{CodecError, StrokeCodec};
use crate::{EncodedPoints, StrokePoints, WorldPos};

stroke_codec_conformance!(Raw, tolerance = 0.0);

/// Offsets that overflow `i32` from the origin split at the last point that still fits.
#[test]
fn splits_where_offset_overflows_i32() {
    let max = i64::from(i32::MAX);
    let points = [at(0, 0), at(max, 0), at(2 * max, 0)];
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
        origin: at(0, 0),
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
fn one_point_budget() {
    let dot = at(-5, 3);
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
    let mut bytes = Raw.encode(&[at(1, 2)], BUDGET).unwrap().remove(0).0;
    bytes.push(0);
    assert_eq!(
        Raw.decode(&EncodedPoints(bytes)),
        Err(CodecError::Malformed)
    );
}

/// Such a piece would overflow `i64` when expanded; `encode` never writes one.
#[test]
fn decode_rejects_offsets_that_overflow_i64() {
    for (origin, offset) in [(at(i64::MAX, 0), [1, 0]), (at(0, i64::MIN), [0, -1])] {
        let piece = StrokePoints {
            origin,
            offsets: vec![offset],
        };
        let bytes = EncodedPoints(postcard::to_allocvec(&piece).unwrap());
        assert_eq!(Raw.decode(&bytes), Err(CodecError::Malformed));
    }
}

/// An origin anywhere, from 1-byte varints near zero to 10-byte ones near the `i64` extremes.
fn origin() -> impl Strategy<Value = WorldPos> {
    let reach = 1i64 << 30;
    let coordinate = prop_oneof![-1000..1000i64, (i64::MIN + reach)..=(i64::MAX - reach)];
    (coordinate.clone(), coordinate).prop_map(|(x, y)| at(x, y))
}

/// Offsets within ±2³⁰, so consecutive steps fit `i32`; small ones and 5-byte ones.
fn offset() -> impl Strategy<Value = [i32; 2]> {
    let axis = prop_oneof![-64..64i32, (1 - (1 << 30))..(1 << 30)];
    [axis.clone(), axis]
}

proptest! {
    /// A piece fills the budget to the byte: the size `Raw` tracks equals postcard's output,
    /// including length prefixes past 127 points and 10-byte origins.
    #[test]
    fn fills_budget_exactly(
        origin in origin(),
        offsets in prop::collection::vec(offset(), 0..300),
    ) {
        let whole = StrokePoints {
            origin,
            offsets: [[0, 0]].into_iter().chain(offsets).collect(),
        };
        let bytes = postcard::to_allocvec(&whole).unwrap();
        let points: Vec<WorldPos> = whole
            .offsets
            .iter()
            .map(|&[dx, dy]| at(origin.x + i64::from(dx), origin.y + i64::from(dy)))
            .collect();

        prop_assert_eq!(Raw.encode(&points, bytes.len()), Ok(vec![EncodedPoints(bytes.clone())]));
        let smaller = Raw.encode(&points, bytes.len() - 1);
        prop_assert!(!matches!(smaller, Ok(pieces) if pieces.len() == 1));
    }
}
