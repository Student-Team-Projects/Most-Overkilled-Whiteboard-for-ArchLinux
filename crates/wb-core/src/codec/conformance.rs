//! Conformance suite for `docs/contracts/stroke-codec.md`, generic over the codec so `Packed`
//! reuses it. Each check names the guarantee it covers; a codec's tests call every check.

use proptest::prelude::*;
use proptest::test_runner::TestRunner;

use super::{CodecError, StrokeCodec};
use crate::{EncodedPoints, StrokePoints, WorldPos};

/// The smallest budget the suite expects every codec to encode any stroke with.
const MIN_BUDGET: usize = 64;

/// Strokes within `encode`'s preconditions: a random walk from anywhere within ±2⁶² whose steps
/// all fit `i32`. Mostly small steps; the rare full-range ones make offsets from a piece's origin
/// overflow `i32` and force splits. 200 steps of at most 2³¹ cannot overflow `i64`.
fn encodable() -> impl Strategy<Value = Vec<WorldPos>> {
    let coord = -(1i64 << 62)..(1i64 << 62);
    let step = prop_oneof![9 => -64..64i64, 1 => any::<i32>().prop_map(i64::from)];
    let steps = prop::collection::vec((step.clone(), step), 0..200);
    (coord.clone(), coord, steps).prop_map(|(x, y, steps)| {
        let mut at = WorldPos { x, y };
        let mut points = vec![at];
        for (dx, dy) in steps {
            at = WorldPos {
                x: at.x + dx,
                y: at.y + dy,
            };
            points.push(at);
        }
        points
    })
}

fn expand(piece: &StrokePoints) -> Vec<WorldPos> {
    piece
        .offsets
        .iter()
        .map(|&[dx, dy]| WorldPos {
            x: piece.origin.x + i64::from(dx),
            y: piece.origin.y + i64::from(dy),
        })
        .collect()
}

/// Checks guarantees 1 and 4 on `encode`'s `Ok` pieces of `points`.
fn check_pieces(
    codec: &impl StrokeCodec,
    points: &[WorldPos],
    budget: usize,
    pieces: &[EncodedPoints],
) -> Result<(), TestCaseError> {
    prop_assert!(!pieces.is_empty());
    let mut joined: Vec<WorldPos> = Vec::new();
    for piece in pieces {
        prop_assert!(piece.0.len() <= budget);
        let decoded = codec
            .decode(piece)
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        let decoded = expand(&decoded);
        match joined.last() {
            None => joined.extend(decoded),
            Some(last) => {
                prop_assert_eq!(Some(last), decoded.first(), "neighbours share an endpoint");
                joined.extend(decoded.into_iter().skip(1));
            }
        }
    }
    prop_assert_eq!(joined, codec.simplify(points));
    Ok(())
}

/// Guarantee 1, on any stroke and budget where `encode` returns `Ok`.
pub(super) fn round_trip_after_simplify(codec: &impl StrokeCodec) {
    let input = (encodable(), 0..1200usize);
    TestRunner::default()
        .run(&input, |(raw, budget)| {
            let points = codec.simplify(&raw);
            match codec.encode(&points, budget) {
                Ok(pieces) => check_pieces(codec, &points, budget, &pieces),
                Err(_) => Ok(()),
            }
        })
        .unwrap();
}

/// Guarantee 2: endpoints kept, every raw point within `tolerance` world units of the result.
pub(super) fn within_tolerance(codec: &impl StrokeCodec, tolerance: f64) {
    TestRunner::default()
        .run(&encodable(), |raw| {
            let simple = codec.simplify(&raw);
            prop_assert_eq!(simple.first(), raw.first());
            prop_assert_eq!(simple.last(), raw.last());
            for &point in &raw {
                prop_assert!(distance_to_path(point, &simple) <= tolerance);
            }
            Ok(())
        })
        .unwrap();
}

/// Guarantee 3: any bytes decode to `Ok` or `Err`, never a panic — random bytes, and every
/// truncation and one-byte corruption of a real piece.
pub(super) fn decode_fuzz(codec: &impl StrokeCodec) {
    TestRunner::default()
        .run(&prop::collection::vec(any::<u8>(), 0..256), |bytes| {
            let _ = codec.decode(&EncodedPoints(bytes));
            Ok(())
        })
        .unwrap();

    let input = (encodable(), any::<prop::sample::Index>(), any::<u8>());
    TestRunner::default()
        .run(&input, |(raw, at, byte)| {
            let pieces = codec
                .encode(&codec.simplify(&raw), 1200)
                .map_err(|e| TestCaseError::fail(e.to_string()))?;
            let bytes = &pieces[0].0;
            for end in 0..bytes.len() {
                let _ = codec.decode(&EncodedPoints(bytes[..end].to_vec()));
            }
            let mut corrupt = bytes.clone();
            corrupt[at.index(bytes.len())] = byte;
            let _ = codec.decode(&EncodedPoints(corrupt));
            Ok(())
        })
        .unwrap();
}

/// Guarantees 1 and 4: within the preconditions `encode` succeeds, and its pieces fit the budget,
/// share endpoints and join back to the simplified stroke — on random strokes and on edge cases.
pub(super) fn encode_split_join_and_pieces_fit_budget(codec: &impl StrokeCodec) {
    let encode_and_check = |raw: &[WorldPos], budget| {
        let points = codec.simplify(raw);
        let pieces = codec
            .encode(&points, budget)
            .map_err(|e| TestCaseError::fail(e.to_string()))?;
        check_pieces(codec, &points, budget, &pieces)
    };
    let input = (encodable(), MIN_BUDGET..1200usize);
    TestRunner::default()
        .run(&input, |(raw, budget)| encode_and_check(&raw, budget))
        .unwrap();

    for raw in edge_strokes() {
        for budget in [MIN_BUDGET, 1200] {
            encode_and_check(&raw, budget).unwrap();
        }
    }
}

/// Encodable strokes at the limits: one point, the `i64` extremes, repeated points, and steps of
/// exactly `i32::MIN` and `i32::MAX`.
fn edge_strokes() -> Vec<Vec<WorldPos>> {
    let at = |x, y| WorldPos { x, y };
    let (min, max) = (i64::MIN, i64::MAX);
    let (step_min, step_max) = (i64::from(i32::MIN), i64::from(i32::MAX));
    vec![
        vec![at(0, 0)],
        vec![at(min, min)],
        vec![at(max, max)],
        vec![at(max - 1, min), at(max, min + 1), at(max, min)],
        vec![at(7, 7); 300],
        vec![
            at(0, 0),
            at(step_max, step_min),
            at(2 * step_max, 2 * step_min),
        ],
        vec![
            at(max, max),
            at(max + step_min, max + step_min),
            at(max - 1, max - 1),
        ],
        (0..300)
            .map(|i| at(i % 2 * step_max, -(i % 2) * step_max))
            .collect(),
    ]
}

/// Guarantee 5: a budget below one piece, and a step too long for `i32` — including one between
/// the `i64` extremes and one after a piece was already filled.
pub(super) fn encode_errors_for_too_small_budget_and_unsplittable_delta(codec: &impl StrokeCodec) {
    let at = |x, y| WorldPos { x, y };
    for budget in [0, 1] {
        assert!(matches!(
            codec.encode(&[at(0, 0), at(10, 10)], budget),
            Err(CodecError::BudgetTooSmall(b)) if b == budget
        ));
    }

    let too_far = i64::from(i32::MAX) + 1;
    let unsplittable = [
        vec![at(0, 0), at(too_far, 0)],
        vec![at(0, 0), at(0, -too_far - 1)],
        vec![at(i64::MIN, i64::MIN), at(i64::MAX, i64::MAX)],
        (0..1000)
            .map(|i| at(i, 0))
            .chain([at(1 << 40, 0)])
            .collect(),
    ];
    for points in unsplittable {
        assert!(matches!(
            codec.encode(&points, 1200),
            Err(CodecError::UnsplittableDelta)
        ));
    }
}

/// Trait docs, not a contract guarantee: an empty stroke has no piece to encode.
pub(super) fn encode_empty_is_err(codec: &impl StrokeCodec) {
    assert!(matches!(codec.encode(&[], 1200), Err(CodecError::NoPoints)));
}

/// Distance from `point` to the polyline `path`, in world units.
fn distance_to_path(point: WorldPos, path: &[WorldPos]) -> f64 {
    // Relative to `point`, so f64 stays exact for the small spans between nearby points.
    #[allow(clippy::cast_precision_loss)] // test-only geometry on spans far below 2⁵³
    let rel = |p: WorldPos| ((p.x - point.x) as f64, (p.y - point.y) as f64);
    let segments = path.windows(2).map(|w| (rel(w[0]), rel(w[1])));
    let single = path.first().map(|&p| (rel(p), rel(p)));
    segments
        .chain(single)
        .map(|((ax, ay), (bx, by))| {
            let (dx, dy) = (bx - ax, by - ay);
            let len2 = dx * dx + dy * dy;
            let t = if len2 == 0.0 {
                0.0
            } else {
                (-(ax * dx + ay * dy) / len2).clamp(0.0, 1.0)
            };
            (ax + t * dx).hypot(ay + t * dy)
        })
        .fold(f64::INFINITY, f64::min)
}
