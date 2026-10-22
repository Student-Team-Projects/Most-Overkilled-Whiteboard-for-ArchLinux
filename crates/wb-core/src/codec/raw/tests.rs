//! `Raw` runs the shared `StrokeCodec` conformance suite, plus checks of its own layout.

use proptest::prelude::*;

use super::super::conformance;
use super::{CodecError, Raw, StrokeCodec};
use crate::{EncodedPoints, StrokePoints, WorldPos};

#[test]
fn round_trip_after_simplify() {
    conformance::round_trip_after_simplify(&Raw);
}

#[test]
fn within_tolerance() {
    conformance::within_tolerance(&Raw, 0.0);
}

#[test]
fn decode_fuzz() {
    conformance::decode_fuzz(&Raw);
}

#[test]
fn encode_split_join_and_pieces_fit_budget() {
    conformance::encode_split_join_and_pieces_fit_budget(&Raw);
}

#[test]
fn encode_errors_for_too_small_budget_and_unsplittable_delta() {
    conformance::encode_errors_for_too_small_budget_and_unsplittable_delta(&Raw);
}

#[test]
fn encode_empty_is_err() {
    conformance::encode_empty_is_err(&Raw);
}

/// Offsets that overflow `i32` from the origin split at the last point that still fits.
#[test]
fn splits_where_offset_overflows_i32() {
    let max = i64::from(i32::MAX);
    let points = [
        WorldPos { x: 0, y: 0 },
        WorldPos { x: max, y: 0 },
        WorldPos { x: 2 * max, y: 0 },
    ];
    let pieces = Raw.encode(&points, 1200).unwrap();
    let decoded: Vec<StrokePoints> = pieces.iter().map(|p| Raw.decode(p).unwrap()).collect();
    assert_eq!(
        decoded,
        [
            StrokePoints {
                origin: points[0],
                offsets: vec![[0, 0], [i32::MAX, 0]],
            },
            StrokePoints {
                origin: points[1],
                offsets: vec![[0, 0], [i32::MAX, 0]],
            },
        ]
    );
}

/// A one-point stroke fits a budget of exactly its own size; a second point then needs a split
/// that cannot happen, since every piece needs two points to make progress.
#[test]
fn one_point_budget() {
    let dot = WorldPos { x: -5, y: 3 };
    let [piece] = &Raw.encode(&[dot], 1200).unwrap()[..] else {
        panic!("one piece")
    };
    let size = piece.0.len();
    assert_eq!(
        Raw.encode(&[dot], size).unwrap(),
        std::slice::from_ref(piece)
    );
    assert!(matches!(
        Raw.encode(&[dot, dot], size),
        Err(CodecError::BudgetTooSmall(b)) if b == size
    ));
}

/// A length prefix far beyond the bytes present is an `Err`, not a huge allocation or a panic.
#[test]
fn decode_rejects_huge_length_prefix() {
    // origin (0, 0), then a varint length of 2⁶³ and no offsets.
    let bytes = [
        0, 0, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01,
    ];
    assert!(Raw.decode(&EncodedPoints(bytes.to_vec())).is_err());
}

proptest! {
    /// A piece fills the budget to the byte: the size `Raw` tracks equals postcard's output.
    #[test]
    fn fills_budget_exactly(
        x in -(1i64 << 62)..(1i64 << 62),
        y in -(1i64 << 62)..(1i64 << 62),
        offsets in prop::collection::vec(any::<[i32; 2]>(), 0..50),
    ) {
        let whole = StrokePoints {
            origin: WorldPos { x, y },
            offsets: [[0, 0]].into_iter().chain(offsets).collect(),
        };
        let points: Vec<WorldPos> = whole
            .offsets
            .iter()
            .map(|&[dx, dy]| WorldPos { x: x + i64::from(dx), y: y + i64::from(dy) })
            .collect();
        let bytes = postcard::to_allocvec(&whole).unwrap();

        let pieces = Raw.encode(&points, bytes.len()).unwrap();
        prop_assert_eq!(pieces, vec![EncodedPoints(bytes.clone())]);
        let smaller = Raw.encode(&points, bytes.len() - 1);
        prop_assert!(!matches!(smaller, Ok(pieces) if pieces.len() == 1));
    }
}
