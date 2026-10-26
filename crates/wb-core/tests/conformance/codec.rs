//! The `StrokeCodec` conformance suite: every law is a function generic over the codec, and
//! `stroke_codec_conformance!` turns each into a test for one codec. Each law names the
//! contract guarantee it checks.

use proptest::prelude::*;
use wb_core::{CodecError, EncodedPoints, StrokeCodec, WorldPos};

use crate::support::codec::{edge_strokes, expand, near_path};
use crate::support::config::{BUDGET, LONG_STROKE, WIDE_OFFSET};

/// Generates `mod conformance` with one test per law for `$codec`. Test parameters: `$tolerance`
/// is the codec's `simplify` tolerance in world units, and `$min_budget` the budget in bytes from
/// which the codec promises that `encode` succeeds on every stroke without an unsplittable step.
macro_rules! stroke_codec_conformance {
    ($codec:expr, tolerance = $tolerance:expr, min_budget = $min_budget:expr) => {
        mod conformance {
            use ::proptest::prelude::*;

            use super::*;
            use $crate::conformance::codec as law;
            use $crate::support::codec::strategy;
            use $crate::support::config::FUZZ_LEN;

            proptest! {
                #[test]
                fn round_trip_after_simplify(
                    raw in strategy::stroke_or_empty(),
                    budget in strategy::any_budget(),
                ) {
                    law::round_trip_after_simplify(&$codec, &raw, budget)?;
                }

                #[test]
                fn within_tolerance(raw in strategy::stroke_or_empty()) {
                    law::within_tolerance(&$codec, $tolerance, &raw)?;
                }

                #[test]
                fn decode_fuzz(bytes in prop::collection::vec(any::<u8>(), 0..FUZZ_LEN)) {
                    law::decode_fuzz(&$codec, bytes);
                }

                #[test]
                fn decode_fuzz_on_damaged_pieces(
                    raw in strategy::stroke(),
                    which in any::<prop::sample::Index>(),
                    cut in any::<prop::sample::Index>(),
                    position in any::<prop::sample::Index>(),
                    byte in any::<u8>(),
                ) {
                    law::decode_fuzz_on_damaged_pieces(
                        &$codec, &raw, [which, cut, position], byte,
                    )?;
                }

                #[test]
                fn encode_split_join_and_pieces_fit_budget(
                    raw in strategy::stroke_or_empty(),
                    budget in strategy::workable_budget($min_budget),
                ) {
                    law::encode_split_join_and_pieces_fit_budget(&$codec, &raw, budget)?;
                }
            }

            #[test]
            fn encode_split_join_on_edge_strokes() -> Result<(), TestCaseError> {
                law::encode_split_join_on_edge_strokes(&$codec, $min_budget)
            }

            #[test]
            fn encode_errors_for_too_small_budget_and_unsplittable_delta()
            -> Result<(), TestCaseError> {
                law::encode_errors_for_too_small_budget_and_unsplittable_delta(&$codec, $min_budget)
            }
        }
    };
}

// A `macro_rules!` macro can be re-exported by path only up to `pub(crate)`.
pub(crate) use stroke_codec_conformance;

/// Guarantees 1 and 4 on `encode`'s pieces of `points`: at least one piece, each within
/// `budget`, each one after the first starting where the previous one ends, joined back into
/// exactly `points`.
fn check_pieces(
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

/// Guarantee 1, at any budget: `Ok` round-trips, and the only `Err` is a too-small budget.
pub fn round_trip_after_simplify(
    codec: &impl StrokeCodec,
    raw: &[WorldPos],
    budget: usize,
) -> Result<(), TestCaseError> {
    let points = codec.simplify(raw);
    match codec.encode(&points, budget) {
        Ok(pieces) => check_pieces(codec, &points, budget, &pieces),
        Err(error) => {
            prop_assert_eq!(error, CodecError::BudgetTooSmall);
            Ok(())
        }
    }
}

/// Guarantee 2: endpoints kept, every raw point within `tolerance` world units of the result;
/// an empty stroke simplifies to an empty one.
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

/// Guarantee 3: any bytes decode to `Ok` or `Err`, never a panic.
pub fn decode_fuzz(codec: &impl StrokeCodec, bytes: Vec<u8>) {
    let _ = codec.decode(&EncodedPoints(bytes));
}

/// Guarantee 3 on bytes close to valid: the piece picked by `which`, cut short at `cut`, and
/// with the byte at `position` replaced by `byte`.
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

/// Guarantees 1 and 4: within `encode`'s preconditions it succeeds, and its pieces fit the
/// budget, share endpoints and join back to the simplified stroke.
pub fn encode_split_join_and_pieces_fit_budget(
    codec: &impl StrokeCodec,
    raw: &[WorldPos],
    budget: usize,
) -> Result<(), TestCaseError> {
    let points = codec.simplify(raw);
    let pieces = codec.encode(&points, budget)?;
    check_pieces(codec, &points, budget, &pieces)
}

/// Guarantees 1 and 4 on strokes at the limits, at the codec's minimum and the real budget.
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

/// Guarantee 5, and `Err` only as it says:
/// - every budget below the smallest that works is `BudgetTooSmall`; that smallest budget fits
///   the stroke exactly as guarantees 1 and 4 require, and is no larger than a single piece;
/// - a consecutive step too long for `i32` is `UnsplittableDelta` at any budget, even where the
///   offsets from the first point would fit.
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
