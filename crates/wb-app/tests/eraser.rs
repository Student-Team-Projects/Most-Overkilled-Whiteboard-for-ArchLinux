//! The eraser's precise hit test.

use wb_app::eraser_utils::touches;
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
    assert!(touches(&segment, 0, WorldPos::new(500, REACH), REACH));
    assert!(!touches(&segment, 0, WorldPos::new(500, REACH + 1), REACH));
}

#[test]
fn reach_past_the_end_is_measured_to_the_end() {
    let segment = line(WorldPos::new(0, 0), &[[0, 0], [1_000, 0]]);
    let three_four_five = WorldPos::new(1_000 + 3 * REACH / 5, 4 * REACH / 5);
    assert!(touches(&segment, 0, three_four_five, REACH));
    assert!(!touches(
        &segment,
        0,
        WorldPos::new(1_000 + REACH + 1, 0),
        REACH
    ));
}

#[test]
fn half_the_width_adds_to_the_reach() {
    let dot = line(WorldPos::new(-7, 9), &[[0, 0]]);
    let width = 40;
    assert!(touches(
        &dot,
        width,
        WorldPos::new(-7 + REACH + 20, 9),
        REACH
    ));
    assert!(!touches(
        &dot,
        width,
        WorldPos::new(-7 + REACH + 21, 9),
        REACH
    ));
}

#[test]
fn a_zero_length_segment_is_measured_to_its_point() {
    let repeated = line(WorldPos::new(0, 0), &[[0, 0], [0, 0]]);
    assert!(touches(&repeated, 0, WorldPos::new(0, REACH), REACH));
    assert!(!touches(&repeated, 0, WorldPos::new(0, REACH + 1), REACH));
}

#[test]
fn no_points_never_touch() {
    assert!(!touches(
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
    assert!(touches(
        &segment,
        0,
        WorldPos::new(far.x + 500, far.y + REACH),
        REACH
    ));
    assert!(!touches(
        &segment,
        0,
        WorldPos::new(far.x + 500, far.y + REACH + 1),
        REACH
    ));
}
