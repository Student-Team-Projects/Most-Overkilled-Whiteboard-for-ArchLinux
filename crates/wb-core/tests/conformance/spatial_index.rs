//! Conformance suite for `SpatialIndex`.

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
            fn insert_replaces_bounds_and_remove_forgets() {
                super::insert_replaces_bounds_and_remove_forgets($new);
            }

            #[test]
            fn touching_boxes_hit_and_gaps_miss() {
                super::touching_boxes_hit_and_gaps_miss($new);
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

/// `query`'s answer, sorted, after checking that it keeps what `out` held and that a second query
/// into the same `out` appends the same ids. Sorting keeps duplicates, so comparing with the
/// model's duplicate-free answer catches them.
fn sorted_query(index: &impl SpatialIndex, area: Aabb) -> Vec<StrokeId> {
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

fn matches_oracle(mut index: impl SpatialIndex, steps: &[Step]) -> Result<(), TestCaseError> {
    let mut model = Model::default();
    for step in steps {
        match *step {
            Step::Insert(id, bbox) => {
                index.insert(id, bbox);
                model.insert(id, bbox);
            }
            Step::Remove(id) => prop_assert_eq!(index.remove(id), model.remove(id)),
            Step::Query(area) => prop_assert_eq!(sorted_query(&index, area), model.query(area)),
        }
    }
    Ok(())
}

fn insert_replaces_bounds_and_remove_forgets(mut index: impl SpatialIndex) {
    let old = Aabb::new(WorldPos::new(0, 0), WorldPos::new(1, 1));
    let new = Aabb::new(WorldPos::new(100, 100), WorldPos::new(101, 101));
    let both = Aabb::new(WorldPos::new(0, 0), WorldPos::new(101, 101));
    index.insert(A, old);
    index.insert(A, new);
    assert_eq!(sorted_query(&index, old), []);
    assert_eq!(sorted_query(&index, both), [A]);
    assert!(index.remove(A));
    assert!(!index.remove(A));
    assert_eq!(sorted_query(&index, both), []);
}

fn touching_boxes_hit_and_gaps_miss(mut index: impl SpatialIndex) {
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
        assert_eq!(sorted_query(&index, area), [A], "{area:?} touches the box");
    }
    let misses = [
        rect(11, 0, 20, 10),
        rect(0, 11, 10, 20),
        rect(11, 11, 20, 20),
        rect(-10, -10, -1, 5),
        rect(-10, -10, 5, -1),
    ];
    for area in misses {
        assert_eq!(sorted_query(&index, area), [], "{area:?} is one unit away");
    }
    assert_eq!(sorted_query(&index, rect(30, 30, 30, 30)), [B]);
}

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

    assert_eq!(sorted_query(&index, point(FAR, FAR)), [A, C]);
    assert_eq!(sorted_query(&index, point(-FAR, -FAR)), [B, C]);
    assert_eq!(sorted_query(&index, point(max, min)), [C, D]);
    assert_eq!(sorted_query(&index, point(max - 1, min + 1)), [C]);
    assert_eq!(sorted_query(&index, point(0, 0)), [C]);
    assert_eq!(sorted_query(&index, world), [A, B, C, D]);

    assert!(index.remove(C));
    assert_eq!(sorted_query(&index, world), [A, B, D]);
}
