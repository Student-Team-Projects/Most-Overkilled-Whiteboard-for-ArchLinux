//! Stroke codec helpers: edge-case strokes, generators, and checks on decoded pieces.

use wb_core::{StrokePoints, WorldPos};

use super::config::LONG_STROKE;

/// The world points of a decoded piece: its origin plus each offset.
pub fn expand(piece: &StrokePoints) -> Vec<WorldPos> {
    let origin = piece.origin;
    piece
        .offsets
        .iter()
        .map(|&[dx, dy]| WorldPos::new(origin.x + i64::from(dx), origin.y + i64::from(dy)))
        .collect()
}

/// Strokes at the limits of `encode`'s preconditions: no point, one point, the `i64` extremes,
/// repeated points, and steps of exactly `i32::MIN` and `i32::MAX`.
pub fn edge_strokes() -> Vec<Vec<WorldPos>> {
    let at = WorldPos::new;
    let (min, max) = (i64::MIN, i64::MAX);
    let (step_min, step_max) = (i64::from(i32::MIN), i64::from(i32::MAX));
    vec![
        vec![],
        vec![at(0, 0)],
        vec![at(min, min)],
        vec![at(max, max)],
        vec![at(max - 1, min), at(max, min + 1), at(max, min)],
        vec![at(7, 7); usize::from(LONG_STROKE)],
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
        (0..i64::from(LONG_STROKE))
            .map(|i| at(i % 2 * step_max, -(i % 2) * step_max))
            .collect(),
    ]
}

/// Whether `point` lies within `tolerance` world units of the polyline `path`.
pub fn near_path(point: WorldPos, path: &[WorldPos], tolerance: f64) -> bool {
    // Relative to `point`, so f64 stays exact for the small spans between nearby points.
    #[allow(clippy::cast_precision_loss)] // only spans far below 2⁵³ are measured
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

/// Generated strokes and budgets.
pub mod strategy {
    use proptest::prelude::*;
    use wb_core::WorldPos;

    use crate::support::config::{BUDGET, LONG_STROKE, MIN_BUDGET, NEAR_ZERO};

    /// A coordinate near zero or anywhere up to the `i64` extremes, from which `steps` full-range
    /// `i32` steps cannot overflow.
    pub fn coordinate(steps: u16) -> impl Strategy<Value = i64> {
        let reach = i64::from(i32::MAX) * i64::from(steps);
        prop_oneof![
            -NEAR_ZERO..NEAR_ZERO,
            (i64::MIN + reach)..=(i64::MAX - reach)
        ]
    }

    /// Strokes within `encode`'s preconditions: random walks whose steps all fit `i32`, from
    /// near the origin or anywhere up to the `i64` extremes. Mostly small steps, so pieces fill
    /// past 127 points; rare full-range steps make offsets overflow `i32` and force splits.
    pub fn stroke() -> impl Strategy<Value = Vec<WorldPos>> {
        let step = prop_oneof![
            99 => -NEAR_ZERO..NEAR_ZERO,
            1 => any::<i32>().prop_map(i64::from),
        ];
        let steps = prop::collection::vec((step.clone(), step), 0..usize::from(LONG_STROKE));
        let start = (coordinate(LONG_STROKE), coordinate(LONG_STROKE));
        (start, steps).prop_map(|((x, y), steps)| {
            let mut point = WorldPos::new(x, y);
            let mut points = vec![point];
            for (dx, dy) in steps {
                point = WorldPos::new(point.x + dx, point.y + dy);
                points.push(point);
            }
            points
        })
    }

    /// Strokes as above, plus the empty stroke.
    pub fn stroke_or_empty() -> impl Strategy<Value = Vec<WorldPos>> {
        prop_oneof![1 => Just(Vec::new()), 9 => stroke()]
    }

    /// Any budget, including ones too small for a single piece.
    pub fn any_budget() -> impl Strategy<Value = usize> {
        0..=BUDGET
    }

    /// Budgets with which every codec must encode any stroke.
    pub fn workable_budget() -> impl Strategy<Value = usize> {
        MIN_BUDGET..=BUDGET
    }
}
