//! `DecodeAll`'s own tests.

use wb_app::{DecodeAll, StrokeCache};
use wb_core::{Aabb, EncodedPoints, Stroke, WorldPos};

use crate::support::stroke_cache::{CountingCodec, StrokeExt};

#[test]
fn decodes_each_stroke_once_wherever_the_screen_is() {
    let codec = CountingCodec::default();
    let mut cache = DecodeAll::new(codec.clone());
    let strokes = [
        Stroke::through(0, &[]).unwrap(),
        Stroke::through(1, &[WorldPos::new(-3, 4)]).unwrap(),
        Stroke::through(2, &[WorldPos::new(0, 0), WorldPos::new(9, 9)]).unwrap(),
    ];
    let far = WorldPos::new(i64::MAX, i64::MAX);
    for area in [
        Aabb::new(WorldPos::new(0, 0), WorldPos::new(1, 1)),
        Aabb::new(far, far),
    ] {
        cache.set_resident(area);
        for stroke in &strokes {
            assert!(cache.get(stroke).is_ok());
        }
    }
    assert_eq!(codec.decodes(), strokes.len());
}

#[test]
fn a_stroke_that_does_not_decode_fails_on_every_get() {
    let codec = CountingCodec::default();
    let mut cache = DecodeAll::new(codec.clone());
    let origin = WorldPos::new(0, 0);
    let junk = Stroke::numbered(0, EncodedPoints(vec![0xFF]), Aabb::new(origin, origin));
    assert!(cache.get(&junk).is_err());
    assert!(cache.get(&junk).is_err());
    assert_eq!(codec.decodes(), 2, "a failed decode is not cached");
}
