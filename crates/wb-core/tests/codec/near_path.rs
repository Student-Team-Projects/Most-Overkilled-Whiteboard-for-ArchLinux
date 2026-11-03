//! The `near_path` test helper against exact cases.

use wb_core::WorldPos;

use crate::support::codec::near_path;

#[test]
fn on_a_vertex_or_inside_a_segment() {
    let path = [WorldPos::new(0, 0), WorldPos::new(10, 0)];
    assert!(near_path(WorldPos::new(10, 0), &path, 0.0));
    assert!(near_path(WorldPos::new(5, 0), &path, 0.0));
}

#[test]
fn beside_a_segment_measures_the_perpendicular_distance() {
    let path = [WorldPos::new(0, 0), WorldPos::new(10, 0)];
    assert!(near_path(WorldPos::new(5, 1), &path, 1.0));
    assert!(!near_path(WorldPos::new(5, 1), &path, 0.99));
}

#[test]
fn past_the_end_of_a_segment_measures_to_the_endpoint() {
    let path = [WorldPos::new(0, 0), WorldPos::new(10, 0)];
    assert!(!near_path(WorldPos::new(12, 0), &path, 1.0));
    assert!(near_path(WorldPos::new(12, 0), &path, 2.0));
}

#[test]
fn single_point_and_empty_paths() {
    let dot = [WorldPos::new(3, 4)];
    assert!(near_path(WorldPos::new(3, 4), &dot, 0.0));
    assert!(near_path(WorldPos::new(0, 0), &dot, 5.0));
    assert!(!near_path(WorldPos::new(0, 0), &dot, 4.99));
    assert!(!near_path(WorldPos::new(0, 0), &[], f64::INFINITY));
}
