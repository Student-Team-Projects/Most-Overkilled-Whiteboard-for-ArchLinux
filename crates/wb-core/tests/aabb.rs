//! `Aabb::covering`: the bounds of a stroke drawn at its width.

use wb_core::{Aabb, StrokePoints, WorldPos};

use crate::support::config::WIDTH;

#[test]
fn covering_a_dot_grows_it_by_half_the_width() {
    let dot = StrokePoints {
        origin: WorldPos::new(-5, 7),
        offsets: vec![[0, 0]],
    };
    let half = i64::from(WIDTH / 2);
    assert_eq!(
        Aabb::covering(&dot, WIDTH),
        Aabb::new(
            WorldPos::new(-5 - half, 7 - half),
            WorldPos::new(-5 + half, 7 + half)
        )
    );
}

#[test]
fn covering_spans_every_offset() {
    let points = StrokePoints {
        origin: WorldPos::new(100, 100),
        offsets: vec![[0, 0], [-30, 5], [12, -40]],
    };
    assert_eq!(
        Aabb::covering(&points, 0),
        Aabb::new(WorldPos::new(70, 60), WorldPos::new(112, 105))
    );
}

#[test]
fn covering_saturates_at_the_edge_of_the_world() {
    let corner = StrokePoints {
        origin: WorldPos::new(i64::MAX, i64::MIN),
        offsets: vec![[i32::MAX, i32::MIN]],
    };
    let bounds = Aabb::covering(&corner, WIDTH);
    assert_eq!(bounds.max.x, i64::MAX);
    assert_eq!(bounds.min.y, i64::MIN);
}
