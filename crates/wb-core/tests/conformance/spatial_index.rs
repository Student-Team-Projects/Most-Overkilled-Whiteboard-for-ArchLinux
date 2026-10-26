//! Conformance suite for `SpatialIndex`. Each law is a generic fn that names the guarantee it
//! checks; `spatial_index_conformance!` turns every law into a test on one impl.

use proptest::prelude::*;
use wb_core::{Aabb, LinearIndex, PeerId, SpatialIndex, StrokeId, WorldPos};

use crate::support::config::FAR;
use crate::support::spatial_index::{Model, Step, session};

/// Generates a module of tests that run every law on a fresh index from `$new`.
macro_rules! spatial_index_conformance {
    ($impl:ident, $new:expr) => {
        mod $impl {
            use super::*;

            proptest! {
                #[test]
                fn matches_oracle(steps in session()) {
                    super::matches_oracle($new, &steps)?;
                }
            }

            #[test]
            fn insert_replaces_bounds() {
                super::insert_replaces_bounds($new);
            }

            #[test]
            fn closed_bounds_touching_boxes_intersect() {
                super::closed_bounds_touching_boxes_intersect($new);
            }

            #[test]
            fn far_coordinates() {
                super::far_coordinates($new);
            }
        }
    };
}

spatial_index_conformance!(linear_index, LinearIndex::new());

const A: StrokeId = StrokeId::new(PeerId(0), 1);
const B: StrokeId = StrokeId::new(PeerId(0), 2);
const C: StrokeId = StrokeId::new(PeerId(0), 3);
const D: StrokeId = StrokeId::new(PeerId(0), 4);

/// An id no law inserts: `out` starts with it, and `query` must leave it there.
const KEPT: StrokeId = StrokeId::new(PeerId(u64::MAX), u64::MAX);

/// `query`'s answer, sorted, after checking it only appends to `out`: what `out` held stays, and
/// a second query into the same `out` appends the same ids again. A duplicate id stays in the
/// sorted answer, so comparing it with a duplicate-free answer catches it.
fn answer(index: &impl SpatialIndex, area: Aabb) -> Vec<StrokeId> {
    let mut out = vec![KEPT];
    index.query(area, &mut out);
    let first_len = out.len();
    index.query(area, &mut out);
    assert_eq!(out.first(), Some(&KEPT), "query keeps what `out` held");
    let mut second = out.split_off(first_len);
    let mut first = out.split_off(1);
    first.sort_unstable();
    second.sort_unstable();
    assert_eq!(
        first, second,
        "a query into a non-empty `out` appends the same ids"
    );
    first
}

/// Guarantees 1 and 2: after every step of a random session, `remove` and `query` answer
/// exactly as the model does.
fn matches_oracle(mut index: impl SpatialIndex, steps: &[Step]) -> Result<(), TestCaseError> {
    let mut model = Model::default();
    for step in steps {
        match *step {
            Step::Insert(id, bbox) => {
                index.insert(id, bbox);
                model.insert(id, bbox);
            }
            Step::Remove(id) => prop_assert_eq!(index.remove(id), model.remove(id)),
            Step::Query(area) => prop_assert_eq!(answer(&index, area), model.query(area)),
        }
    }
    Ok(())
}

/// Guarantee 2: a second insert of an id moves it; it is found once, only at its new bounds, and
/// one remove takes it out.
fn insert_replaces_bounds(mut index: impl SpatialIndex) {
    let old = Aabb::new(WorldPos::new(0, 0), WorldPos::new(1, 1));
    let new = Aabb::new(WorldPos::new(100, 100), WorldPos::new(101, 101));
    let both = Aabb::new(WorldPos::new(0, 0), WorldPos::new(101, 101));
    index.insert(A, old);
    index.insert(A, new);
    assert_eq!(answer(&index, old), []);
    assert_eq!(answer(&index, both), [A]);
    assert!(index.remove(A));
    assert!(!index.remove(A));
    assert_eq!(answer(&index, both), []);
}

/// Guarantee 1 at the edges: bounds are closed, so sharing an edge or a corner hits, and one
/// unit apart misses.
fn closed_bounds_touching_boxes_intersect(mut index: impl SpatialIndex) {
    let rect = |x0, y0, x1, y1| Aabb::new(WorldPos::new(x0, y0), WorldPos::new(x1, y1));
    index.insert(A, rect(0, 0, 10, 10));
    index.insert(B, rect(30, 30, 30, 30));
    let hits = [
        rect(10, 0, 20, 10),
        rect(0, 10, 10, 20),
        rect(-10, -10, 0, 0),
        rect(5, 5, 5, 5),
        rect(-5, -5, 15, 15),
    ];
    for area in hits {
        assert_eq!(answer(&index, area), [A], "{area:?} touches the box");
    }
    let misses = [
        rect(11, 0, 20, 10),
        rect(0, 11, 10, 20),
        rect(11, 11, 20, 20),
        rect(-10, -10, -1, 5),
        rect(-10, -10, 5, -1),
    ];
    for area in misses {
        assert_eq!(answer(&index, area), [], "{area:?} is one unit away");
    }
    assert_eq!(answer(&index, rect(30, 30, 30, 30)), [B]);
}

/// Guarantee 3: no fixed world bounds, from ±2⁶² to the `i64` extremes.
fn far_coordinates(mut index: impl SpatialIndex) {
    let (min, max) = (i64::MIN, i64::MAX);
    let point = |x, y| Aabb::new(WorldPos::new(x, y), WorldPos::new(x, y));
    let world = Aabb::new(WorldPos::new(min, min), WorldPos::new(max, max));
    index.insert(
        A,
        Aabb::new(WorldPos::new(FAR, FAR), WorldPos::new(FAR + 10, FAR + 10)),
    );
    index.insert(
        B,
        Aabb::new(
            WorldPos::new(-FAR - 10, -FAR - 10),
            WorldPos::new(-FAR, -FAR),
        ),
    );
    index.insert(C, world);
    index.insert(D, point(max, min));

    assert_eq!(answer(&index, point(FAR, FAR)), [A, C]);
    assert_eq!(answer(&index, point(-FAR, -FAR)), [B, C]);
    assert_eq!(answer(&index, point(max, min)), [C, D]);
    assert_eq!(answer(&index, point(max - 1, min + 1)), [C]);
    assert_eq!(answer(&index, point(0, 0)), [C]);
    assert_eq!(answer(&index, world), [A, B, C, D]);

    assert!(index.remove(C));
    assert_eq!(answer(&index, world), [A, B, D]);
}
