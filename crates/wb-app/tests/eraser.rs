//! The eraser's precise hit test.

use wb_app::eraser_utils::stroke_is_within_eraser_reach;
use wb_core::{StrokePoints, WorldPos};

use crate::support::config::REACH;

fn line(origin: WorldPos, offsets: &[[i32; 2]]) -> StrokePoints {
    StrokePoints {
        origin,
        offsets: offsets.to_vec(),
    }
}

#[test]
fn reach_beside_a_segment_is_measured_straight_across() {
    let segment = line(WorldPos::new(0, 0), &[[0, 0], [1_000, 0]]);
    assert!(stroke_is_within_eraser_reach(
        &segment,
        0,
        WorldPos::new(500, i64::from(REACH)),
        REACH
    ));
    assert!(!stroke_is_within_eraser_reach(
        &segment,
        0,
        WorldPos::new(500, i64::from(REACH) + 1),
        REACH
    ));
}

#[test]
fn reach_past_the_end_is_measured_to_the_end() {
    let segment = line(WorldPos::new(0, 0), &[[0, 0], [1_000, 0]]);
    let three_four_five = WorldPos::new(1_000 + 3 * i64::from(REACH) / 5, 4 * i64::from(REACH) / 5);
    assert!(stroke_is_within_eraser_reach(
        &segment,
        0,
        three_four_five,
        REACH
    ));
    assert!(!stroke_is_within_eraser_reach(
        &segment,
        0,
        WorldPos::new(1_000 + i64::from(REACH) + 1, 0),
        REACH
    ));
}

#[test]
fn half_the_width_adds_to_the_reach() {
    let dot = line(WorldPos::new(-7, 9), &[[0, 0]]);
    let width = 40;
    assert!(stroke_is_within_eraser_reach(
        &dot,
        width,
        WorldPos::new(-7 + i64::from(REACH) + 20, 9),
        REACH
    ));
    assert!(!stroke_is_within_eraser_reach(
        &dot,
        width,
        WorldPos::new(-7 + i64::from(REACH) + 21, 9),
        REACH
    ));
}

#[test]
fn a_zero_length_segment_is_measured_to_its_point() {
    let repeated = line(WorldPos::new(0, 0), &[[0, 0], [0, 0]]);
    assert!(stroke_is_within_eraser_reach(
        &repeated,
        0,
        WorldPos::new(0, i64::from(REACH)),
        REACH
    ));
    assert!(!stroke_is_within_eraser_reach(
        &repeated,
        0,
        WorldPos::new(0, i64::from(REACH) + 1),
        REACH
    ));
}

#[test]
fn no_points_never_touch() {
    assert!(!stroke_is_within_eraser_reach(
        &line(WorldPos::new(0, 0), &[]),
        0,
        WorldPos::new(0, 0),
        REACH
    ));
}

#[test]
fn far_from_the_origin_the_test_stays_exact() {
    let far = WorldPos::new(i64::MAX - 10_000, i64::MIN + 10_000);
    let segment = line(far, &[[0, 0], [1_000, 0]]);
    assert!(stroke_is_within_eraser_reach(
        &segment,
        0,
        WorldPos::new(far.x + 500, far.y + i64::from(REACH)),
        REACH
    ));
    assert!(!stroke_is_within_eraser_reach(
        &segment,
        0,
        WorldPos::new(far.x + 500, far.y + i64::from(REACH) + 1),
        REACH
    ));
}
