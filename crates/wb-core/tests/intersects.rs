//! `Intersects` on `Aabb`: closed bounds, symmetric.

use wb_core::{Aabb, Intersects, WorldPos};

use crate::support::config::UNIT;

/// A `UNIT`-sided box with its min corner at `(x, y)`.
const fn unit(x: i64, y: i64) -> Aabb {
    Aabb::new(WorldPos::new(x, y), WorldPos::new(x + UNIT, y + UNIT))
}

#[test]
fn aabb_intersects_on_shared_corner_not_on_gap() {
    let base = unit(0, 0);
    assert!(base.intersects(&unit(UNIT, UNIT)), "shared corner");
    assert!(unit(UNIT, 0).intersects(&base), "shared edge, reversed");
    assert!(!base.intersects(&unit(UNIT + 1, 0)), "gap on x");
    assert!(!base.intersects(&unit(0, -UNIT - 1)), "gap on y");
}
