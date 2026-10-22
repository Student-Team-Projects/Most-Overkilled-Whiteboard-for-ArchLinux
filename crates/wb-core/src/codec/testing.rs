//! Test API for stroke codecs: budgets, generated strokes, edge-case strokes and piece helpers.

use crate::{StrokePoints, WorldPos};

/// The real piece budget is about one datagram (`transport.md`: 1200 bytes).
pub(crate) const BUDGET: usize = 1200;

/// Smallest budget every codec must encode any stroke with: Raw's largest two-point piece is
/// 33 bytes, so 64 leaves room for any codec's header.
pub(crate) const MIN_BUDGET: usize = 64;

pub(crate) fn at(x: i64, y: i64) -> WorldPos {
    WorldPos { x, y }
}

/// The world points of a decoded piece.
pub(crate) fn expand(piece: &StrokePoints) -> Vec<WorldPos> {
    piece
        .offsets
        .iter()
        .map(|&[dx, dy]| {
            at(
                piece.origin.x + i64::from(dx),
                piece.origin.y + i64::from(dy),
            )
        })
        .collect()
}

/// Strokes at the limits of `encode`'s preconditions: no point, one point, the `i64` extremes,
/// repeated points, and steps of exactly `i32::MIN` and `i32::MAX`.
pub(crate) fn edge_strokes() -> Vec<Vec<WorldPos>> {
    let (min, max) = (i64::MIN, i64::MAX);
    let (step_min, step_max) = (i64::from(i32::MIN), i64::from(i32::MAX));
    vec![
        vec![],
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

/// Whether `point` lies within `tolerance` world units of the polyline `path`.
pub(crate) fn near_path(point: WorldPos, path: &[WorldPos], tolerance: f64) -> bool {
    // Relative to `point`, so f64 stays exact for the small spans between nearby points.
    #[allow(clippy::cast_precision_loss)] // test-only geometry on spans far below 2⁵³
    let rel = |p: WorldPos| ((p.x - point.x) as f64, (p.y - point.y) as f64);
    let segments = path.windows(2).map(|w| (rel(w[0]), rel(w[1])));
    let single = path.first().map(|&p| (rel(p), rel(p)));
    segments.chain(single).any(|((ax, ay), (bx, by))| {
        let (dx, dy) = (bx - ax, by - ay);
        let len2 = dx * dx + dy * dy;
        let t = if len2 == 0.0 {
            0.0
        } else {
            (-(ax * dx + ay * dy) / len2).clamp(0.0, 1.0)
        };
        (ax + t * dx).hypot(ay + t * dy) <= tolerance
    })
}

pub(crate) mod strategy {
    use proptest::prelude::*;

    use super::{BUDGET, MIN_BUDGET, at};
    use crate::WorldPos;

    /// Any coordinate a walk of `steps` full-range `i32` steps can start from without overflow.
    fn coordinate(steps: usize) -> impl Strategy<Value = i64> {
        let reach = i64::from(i32::MAX) * i64::try_from(steps).unwrap_or(i64::MAX);
        prop_oneof![-1000..1000i64, (i64::MIN + reach)..=(i64::MAX - reach)]
    }

    /// Strokes within `encode`'s preconditions: random walks whose steps all fit `i32`, from
    /// near the origin or anywhere up to the `i64` extremes. Mostly small steps, so pieces fill
    /// past 128 points; rare full-range steps make offsets overflow `i32` and force splits.
    pub(crate) fn stroke() -> impl Strategy<Value = Vec<WorldPos>> {
        const MAX_STEPS: usize = 300;
        let step = prop_oneof![99 => -64..64i64, 1 => any::<i32>().prop_map(i64::from)];
        let steps = prop::collection::vec((step.clone(), step), 0..MAX_STEPS);
        (coordinate(MAX_STEPS), coordinate(MAX_STEPS), steps).prop_map(|(x, y, steps)| {
            let mut point = at(x, y);
            let mut points = vec![point];
            for (dx, dy) in steps {
                point = at(point.x + dx, point.y + dy);
                points.push(point);
            }
            points
        })
    }

    /// Strokes as above, plus the empty stroke.
    pub(crate) fn stroke_or_empty() -> impl Strategy<Value = Vec<WorldPos>> {
        prop_oneof![1 => Just(Vec::new()), 9 => stroke()]
    }

    /// Any budget, including ones too small for a single piece.
    pub(crate) fn any_budget() -> impl Strategy<Value = usize> {
        0..=BUDGET
    }

    /// Budgets every codec must encode any stroke with.
    pub(crate) fn budget() -> impl Strategy<Value = usize> {
        MIN_BUDGET..=BUDGET
    }
}
