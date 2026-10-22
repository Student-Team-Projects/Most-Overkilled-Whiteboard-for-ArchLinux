//! `near_path` decides guarantee 2 for every codec, so it is checked against exact cases.

use super::{at, near_path};

#[test]
fn on_a_vertex_or_inside_a_segment() {
    let path = [at(0, 0), at(10, 0)];
    assert!(near_path(at(10, 0), &path, 0.0));
    assert!(near_path(at(5, 0), &path, 0.0));
}

#[test]
fn off_a_segment_by_its_perpendicular_distance() {
    let path = [at(0, 0), at(10, 0)];
    assert!(near_path(at(5, 1), &path, 1.0));
    assert!(!near_path(at(5, 1), &path, 0.99));
}

/// Past a segment's end the distance is to the endpoint, not to the infinite line.
#[test]
fn past_the_end_of_a_segment() {
    let path = [at(0, 0), at(10, 0)];
    assert!(!near_path(at(12, 0), &path, 1.0));
    assert!(near_path(at(12, 0), &path, 2.0));
}

#[test]
fn single_point_and_empty_paths() {
    let dot = [at(3, 4)];
    assert!(near_path(at(3, 4), &dot, 0.0));
    assert!(near_path(at(0, 0), &dot, 5.0));
    assert!(!near_path(at(0, 0), &dot, 4.99));
    assert!(!near_path(at(0, 0), &[], f64::INFINITY));
}
