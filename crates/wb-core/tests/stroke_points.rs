//! `StrokePoints` into world points.

use wb_core::{StrokePoints, WorldPos};

#[test]
fn world_points_are_the_origin_plus_each_offset_in_order() {
    let points = StrokePoints {
        origin: WorldPos::new(100, -100),
        offsets: vec![[0, 0], [-30, 5], [12, -40]],
    };
    assert_eq!(
        Vec::from(&points),
        [
            WorldPos::new(100, -100),
            WorldPos::new(70, -95),
            WorldPos::new(112, -140)
        ]
    );
}

#[test]
fn world_points_saturate_at_the_edge_of_the_world() {
    let corner = StrokePoints {
        origin: WorldPos::new(i64::MAX, i64::MIN),
        offsets: vec![[i32::MAX, i32::MIN]],
    };
    assert_eq!(Vec::from(&corner), [WorldPos::new(i64::MAX, i64::MIN)]);
}
