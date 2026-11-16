//! Conformance suite for `StrokeCache`.

use proptest::prelude::*;
use wb_app::{DecodeAll, StrokeCache};
use wb_core::{Raw, Stroke, StrokeCodec, WorldPos};

use crate::support::stroke_cache::{CountingCodec, Step, StrokeExt, board, session};

/// Generates a module of tests that run every law on a cache from `$new`, a function from the
/// codec to decode with to a fresh cache.
macro_rules! stroke_cache_conformance {
    ($impl:ident, $new:expr) => {
        mod $impl {
            use super::*;

            proptest! {
                #[test]
                fn matches_decode(strokes in board(), steps in session()) {
                    super::matches_decode($new(CountingCodec::default()), &strokes, &steps)?;
                }
            }

            #[test]
            fn forget_drops() -> Result<(), TestCaseError> {
                let codec = CountingCodec::default();
                super::forget_drops($new(codec.clone()), &codec)
            }
        }
    };
}

stroke_cache_conformance!(decode_all, DecodeAll::new);

fn matches_decode(
    mut cache: impl StrokeCache,
    strokes: &[Stroke],
    steps: &[Step],
) -> Result<(), TestCaseError> {
    for step in steps {
        match step {
            Step::Get(which) => {
                let stroke = which.get(strokes);
                prop_assert_eq!(cache.get(stroke).cloned(), Raw.decode(&stroke.points));
            }
            Step::SetResident(area) => cache.set_resident(*area),
            Step::Forget(which) => cache.forget(which.get(strokes).id),
        }
    }
    Ok(())
}

/// A get after `forget` has to decode again, so the cache kept nothing for the id.
fn forget_drops(mut cache: impl StrokeCache, codec: &CountingCodec) -> Result<(), TestCaseError> {
    let line = Stroke::through(0, &[WorldPos::new(0, 0), WorldPos::new(5, -5)])
        .ok_or_else(|| TestCaseError::fail("a short line fits one piece"))?;
    let expected = Raw.decode(&line.points);
    prop_assert_eq!(cache.get(&line).cloned(), expected.clone());
    cache.forget(line.id);
    let before = codec.decodes();
    prop_assert_eq!(cache.get(&line).cloned(), expected);
    prop_assert_eq!(
        codec.decodes(),
        before + 1,
        "the forgotten stroke is decoded again"
    );
    Ok(())
}
