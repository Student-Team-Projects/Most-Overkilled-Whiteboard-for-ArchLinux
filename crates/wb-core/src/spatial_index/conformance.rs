//! Conformance suite for `docs/contracts/spatial-index.md`, generic over the index so `QuadTree`
//! reuses it. `spatial_index_conformance!` generates one test per law; each law names the
//! guarantee it checks.

use proptest::prelude::*;

use super::SpatialIndex;
use super::testing::{Model, Step, aabb, id};
use crate::{Aabb, OpId, PeerId, StrokeId};

/// Generates `mod conformance` with one test per law, each on a fresh index from `$new`.
macro_rules! spatial_index_conformance {
    ($new:expr) => {
        mod conformance {
            use ::proptest::prelude::*;

            use super::*;
            use $crate::spatial_index::conformance as law;
            use $crate::spatial_index::testing::strategy;

            proptest! {
                #[test]
                fn matches_oracle(steps in strategy::session()) {
                    law::matches_oracle($new, &steps)?;
                }
            }

            #[test]
            fn insert_replaces_bounds() {
                law::insert_replaces_bounds($new);
            }

            #[test]
            fn closed_bounds_touching_boxes_intersect() {
                law::closed_bounds_touching_boxes_intersect($new);
            }

            #[test]
            fn far_coordinates() {
                law::far_coordinates($new);
            }
        }
    };
}

pub(crate) use spatial_index_conformance;

/// An id no law inserts: `out` starts with it, and `query` must leave it there.
const KEPT: StrokeId = OpId {
    peer: PeerId(u64::MAX),
    seq: u64::MAX,
};

/// `query`'s answer, sorted, after checking it only appended to `out`. A duplicate id stays in
/// the sorted answer, so comparing it with a set catches duplicates too.
fn answer(index: &impl SpatialIndex, area: Aabb) -> Vec<StrokeId> {
    let mut out = vec![KEPT];
    index.query(area, &mut out);
    assert_eq!(out.first(), Some(&KEPT), "query keeps what `out` held");
    let mut answer = out.split_off(1);
    answer.sort_unstable();
    answer
}

/// Guarantees 1 and 2: after every step of a random session, `remove` and `query` answer
/// exactly as the model does.
pub fn matches_oracle(mut index: impl SpatialIndex, steps: &[Step]) -> Result<(), TestCaseError> {
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

/// Guarantee 2: a second insert of an id moves it; it is found once, only at its new bounds.
pub fn insert_replaces_bounds(mut index: impl SpatialIndex) {
    index.insert(id(1), aabb(0, 0, 1, 1));
    index.insert(id(1), aabb(100, 100, 101, 101));
    assert_eq!(answer(&index, aabb(0, 0, 1, 1)), []);
    assert_eq!(answer(&index, aabb(0, 0, 101, 101)), [id(1)]);
    assert!(index.remove(id(1)));
    assert!(!index.remove(id(1)));
}

/// Guarantee 1 at the edges: bounds are closed, so sharing an edge or a corner hits, and one
/// unit apart misses.
pub fn closed_bounds_touching_boxes_intersect(mut index: impl SpatialIndex) {
    index.insert(id(1), aabb(0, 0, 10, 10));
    index.insert(id(2), aabb(30, 30, 30, 30));
    let hits = [
        aabb(10, 0, 20, 10),
        aabb(-10, -10, 0, 0),
        aabb(5, 5, 5, 5),
        aabb(-5, -5, 15, 15),
    ];
    for area in hits {
        assert_eq!(answer(&index, area), [id(1)], "{area:?} touches the box");
    }
    let misses = [
        aabb(11, 0, 20, 10),
        aabb(0, 11, 10, 20),
        aabb(11, 11, 20, 20),
    ];
    for area in misses {
        assert_eq!(answer(&index, area), [], "{area:?} is one unit away");
    }
    assert_eq!(answer(&index, aabb(30, 30, 30, 30)), [id(2)]);
}

/// Guarantee 3: no fixed world bounds, from ±2⁶² to the `i64` extremes.
pub fn far_coordinates(mut index: impl SpatialIndex) {
    let (far, min, max) = (1i64 << 62, i64::MIN, i64::MAX);
    index.insert(id(1), aabb(far, far, far + 10, far + 10));
    index.insert(id(2), aabb(-far - 10, -far - 10, -far, -far));
    index.insert(id(3), aabb(min, min, max, max));
    index.insert(id(4), aabb(max, min, max, min));

    assert_eq!(answer(&index, aabb(far, far, far, far)), [id(1), id(3)]);
    assert_eq!(answer(&index, aabb(-far, -far, -far, -far)), [id(2), id(3)]);
    assert_eq!(answer(&index, aabb(max, min, max, min)), [id(3), id(4)]);
    assert_eq!(answer(&index, aabb(0, 0, 1, 1)), [id(3)]);
    assert_eq!(
        answer(&index, aabb(min, min, max, max)),
        [id(1), id(2), id(3), id(4)]
    );

    assert!(index.remove(id(3)));
    assert_eq!(
        answer(&index, aabb(min, min, max, max)),
        [id(1), id(2), id(4)]
    );
}
