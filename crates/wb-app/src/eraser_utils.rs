//! The eraser's precise hit test, after the spatial index has picked the candidates.

use wb_core::{StrokePoints, WorldPos};

/// Whether a stroke through `points`, drawn `width` world units wide, comes within the eraser's
/// `reach` world units of `centre`: some segment, or the only point of a dot, lies at most
/// `reach + width / 2` from it. A stroke with no points is never within reach.
///
/// Coordinates are taken relative to `centre` in `i64` first, then measured in `f64`, which is
/// exact for every point closer than 2^53 world units; farther points are far out of reach.
#[must_use]
#[allow(clippy::cast_precision_loss)] // offsets are exact below 2^53, and anything farther is out of reach
pub fn stroke_is_within_eraser_reach(
    points: &StrokePoints,
    width: u32,
    centre: WorldPos,
    reach: u32,
) -> bool {
    let limit = f64::from(reach.saturating_add(width / 2));
    let relative: Vec<[f64; 2]> = Vec::<WorldPos>::from(points)
        .into_iter()
        .map(|point| {
            [
                point.x.saturating_sub(centre.x) as f64,
                point.y.saturating_sub(centre.y) as f64,
            ]
        })
        .collect();
    match relative.as_slice() {
        [] => false,
        [[x, y]] => x.hypot(*y) <= limit,
        segments => segments
            .windows(2)
            .any(|segment| distance_from_origin_to_segment(segment[0], segment[1]) <= limit),
    }
}

/// Distance from `(0, 0)`, the eraser centre once points are taken relative to it, to the
/// segment from `start` to `end`.
///
/// Projects `(0, 0)` onto the line through the segment, clamps the projection to the segment's
/// ends, and measures to that closest point. A segment of zero length is measured to its point.
fn distance_from_origin_to_segment(start: [f64; 2], end: [f64; 2]) -> f64 {
    let along = [end[0] - start[0], end[1] - start[1]];
    let length_squared = along[0] * along[0] + along[1] * along[1];
    let closest_fraction = if length_squared > 0.0 {
        (-(start[0] * along[0] + start[1] * along[1]) / length_squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let closest = [
        start[0] + along[0] * closest_fraction,
        start[1] + along[1] * closest_fraction,
    ];
    closest[0].hypot(closest[1])
}
