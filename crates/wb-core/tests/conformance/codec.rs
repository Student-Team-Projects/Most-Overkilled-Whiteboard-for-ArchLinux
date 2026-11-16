//! `StrokeCodec` laws, generic over the codec; `stroke_codec_conformance!` runs them on one.

use proptest::prelude::*;
use wb_core::{CodecError, EncodedPoints, Raw, StrokeCodec, WorldPos};

use crate::support::codec::{edge_strokes, expand, near_path};
use crate::support::config::{BUDGET, LONG_STROKE, RAW_MIN_BUDGET, WIDE_OFFSET};

/// Generates one test per law for `$codec`. `$tolerance` is its `simplify` tolerance in world
/// units; `$min_budget` a budget from which `encode` must succeed on every stroke whose
/// steps fit `i32`.
macro_rules! stroke_codec_conformance {
    ($name:ident, $codec:expr, tolerance = $tolerance:expr, min_budget = $min_budget:expr) => {
        mod $name {
            use ::proptest::prelude::*;

            use super::*;
            use $crate::support::codec::strategy;
            use $crate::support::config::FUZZ_LEN;

            proptest! {
                #[test]
                fn round_trip_after_simplify(
                    raw in strategy::stroke_or_empty(),
                    budget in strategy::any_budget(),
                ) {
                    super::round_trip_after_simplify(&$codec, &raw, budget)?;
                }

                #[test]
                fn within_tolerance(raw in strategy::stroke_or_empty()) {
                    super::within_tolerance(&$codec, $tolerance, &raw)?;
                }

                #[test]
                fn decode_fuzz(bytes in prop::collection::vec(any::<u8>(), 0..FUZZ_LEN)) {
                    super::decode_fuzz(&$codec, bytes);
                }

                #[test]
                fn decode_fuzz_on_damaged_pieces(
                    raw in strategy::stroke(),
                    which in any::<prop::sample::Index>(),
                    cut in any::<prop::sample::Index>(),
                    position in any::<prop::sample::Index>(),
                    byte in any::<u8>(),
                ) {
                    super::decode_fuzz_on_damaged_pieces(
                        &$codec, &raw, [which, cut, position], byte,
                    )?;
                }

                #[test]
                fn encode_split_join_and_pieces_fit_budget(
                    raw in strategy::stroke_or_empty(),
                    budget in strategy::workable_budget($min_budget),
                ) {
                    super::encode_split_join_and_pieces_fit_budget(&$codec, &raw, budget)?;
                }
            }

            #[test]
            fn encode_split_join_on_edge_strokes() -> Result<(), TestCaseError> {
                super::encode_split_join_on_edge_strokes(&$codec, $min_budget)
            }

            #[test]
            fn encode_errors_for_too_small_budget_and_unsplittable_delta()
            -> Result<(), TestCaseError> {
                super::encode_errors_for_too_small_budget_and_unsplittable_delta(
                    &$codec,
                    $min_budget,
                )
            }
        }
    };
}

stroke_codec_conformance!(raw, Raw, tolerance = 0.0, min_budget = RAW_MIN_BUDGET);

fn check_pieces_fit_and_join(
    codec: &impl StrokeCodec,
    points: &[WorldPos],
    budget: usize,
    pieces: &[EncodedPoints],
) -> Result<(), TestCaseError> {
    prop_assert!(!pieces.is_empty());
    let mut decoded = Vec::new();
    for piece in pieces {
        prop_assert!(piece.0.len() <= budget);
        let points = expand(&codec.decode(piece)?)
            .ok_or_else(|| TestCaseError::fail("decoded offsets overflow i64"))?;
        decoded.push(points);
    }
    for pair in decoded.windows(2) {
        let shared = pair[0]
            .last()
            .is_some_and(|end| pair[1].first() == Some(end));
        prop_assert!(shared, "neighbours share an endpoint");
    }
    let joined: Vec<WorldPos> = decoded
        .iter()
        .enumerate()
        .flat_map(|(i, piece)| piece.iter().skip(usize::from(i > 0)))
        .copied()
        .collect();
    prop_assert_eq!(joined, points);
    Ok(())
}

/// At any budget the only error is a too-small budget.
pub fn round_trip_after_simplify(
    codec: &impl StrokeCodec,
    raw: &[WorldPos],
    budget: usize,
) -> Result<(), TestCaseError> {
    let points = codec.simplify(raw);
    match codec.encode(&points, budget) {
        Ok(pieces) => check_pieces_fit_and_join(codec, &points, budget, &pieces),
        Err(error) => {
            prop_assert_eq!(error, CodecError::BudgetTooSmall);
            Ok(())
        }
    }
}

pub fn within_tolerance(
    codec: &impl StrokeCodec,
    tolerance: f64,
    raw: &[WorldPos],
) -> Result<(), TestCaseError> {
    let simple = codec.simplify(raw);
    prop_assert_eq!(simple.first(), raw.first());
    prop_assert_eq!(simple.last(), raw.last());
    for &point in raw {
        prop_assert!(
            near_path(point, &simple, tolerance),
            "{point:?} is off the path"
        );
    }
    Ok(())
}

pub fn decode_fuzz(codec: &impl StrokeCodec, bytes: Vec<u8>) {
    let _ = codec.decode(&EncodedPoints(bytes));
}

/// Decodes one of `raw`'s pieces cut short at `cut`, and the same piece with the byte at
/// `position` replaced by `byte`.
pub fn decode_fuzz_on_damaged_pieces(
    codec: &impl StrokeCodec,
    raw: &[WorldPos],
    [which, cut, position]: [prop::sample::Index; 3],
    byte: u8,
) -> Result<(), TestCaseError> {
    let pieces = codec.encode(&codec.simplify(raw), BUDGET)?;
    prop_assert!(
        !pieces.is_empty(),
        "a successful encode returns at least one piece"
    );
    let bytes = &which.get(&pieces).0;
    if !bytes.is_empty() {
        decode_fuzz(codec, bytes[..cut.index(bytes.len())].to_vec());
        let mut corrupted = bytes.clone();
        corrupted[position.index(bytes.len())] = byte;
        decode_fuzz(codec, corrupted);
    }
    Ok(())
}

pub fn encode_split_join_and_pieces_fit_budget(
    codec: &impl StrokeCodec,
    raw: &[WorldPos],
    budget: usize,
) -> Result<(), TestCaseError> {
    let points = codec.simplify(raw);
    let pieces = codec.encode(&points, budget)?;
    check_pieces_fit_and_join(codec, &points, budget, &pieces)
}

pub fn encode_split_join_on_edge_strokes(
    codec: &impl StrokeCodec,
    min_budget: usize,
) -> Result<(), TestCaseError> {
    for raw in edge_strokes() {
        for budget in [min_budget, BUDGET] {
            encode_split_join_and_pieces_fit_budget(codec, &raw, budget)
                .map_err(|e| TestCaseError::fail(format!("{raw:?} at budget {budget}: {e}")))?;
        }
    }
    Ok(())
}

/// Every budget below the smallest that works is `BudgetTooSmall`, and that smallest is no
/// larger than the stroke's single piece. A step too long for `i32` is `UnsplittableDelta` at
/// any budget, even where the offsets from the first point would fit.
pub fn encode_errors_for_too_small_budget_and_unsplittable_delta(
    codec: &impl StrokeCodec,
    min_budget: usize,
) -> Result<(), TestCaseError> {
    let fitting = [
        vec![],
        vec![WorldPos::new(0, 0)],
        vec![WorldPos::new(0, 0), WorldPos::new(10, 10)],
        vec![WorldPos::new(i64::MIN, 5), WorldPos::new(i64::MIN, -5)],
    ];
    for raw in fitting {
        let points = codec.simplify(&raw);
        let smallest = (0..=BUDGET)
            .find(|&budget| codec.encode(&points, budget).is_ok())
            .ok_or_else(|| TestCaseError::fail(format!("{raw:?} fits no budget up to {BUDGET}")))?;
        encode_split_join_and_pieces_fit_budget(codec, &raw, smallest).map_err(|e| {
            TestCaseError::fail(format!("{raw:?} at its smallest budget {smallest}: {e}"))
        })?;
        for budget in 0..smallest {
            prop_assert_eq!(
                codec.encode(&points, budget),
                Err(CodecError::BudgetTooSmall)
            );
        }
        if let Ok([piece]) = codec.encode(&points, BUDGET).as_deref() {
            prop_assert!(
                smallest <= piece.0.len(),
                "{raw:?} fits one piece but needs a larger budget"
            );
        }
    }

    let too_far = i64::from(i32::MAX) + 1;
    let unsplittable = [
        vec![WorldPos::new(0, 0), WorldPos::new(too_far, 0)],
        vec![WorldPos::new(0, 0), WorldPos::new(0, -too_far - 1)],
        vec![
            WorldPos::new(i64::MIN, i64::MIN),
            WorldPos::new(i64::MAX, i64::MAX),
        ],
        vec![
            WorldPos::new(0, 0),
            WorldPos::new(-i64::from(WIDE_OFFSET), 0),
            WorldPos::new(i64::from(WIDE_OFFSET), 0),
        ],
        (0..i64::from(LONG_STROKE))
            .map(|i| WorldPos::new(i, 0))
            .chain([WorldPos::new(2 * too_far, 0)])
            .collect(),
    ];
    for raw in unsplittable {
        let points = codec.simplify(&raw);
        for budget in [0, min_budget, BUDGET, usize::MAX] {
            prop_assert_eq!(
                codec.encode(&points, budget),
                Err(CodecError::UnsplittableDelta)
            );
        }
    }
    Ok(())
}
