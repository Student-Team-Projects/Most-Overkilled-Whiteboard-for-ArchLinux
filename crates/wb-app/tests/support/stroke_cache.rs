//! Strokes, sessions and an instrumented codec for the `StrokeCache` tests.

use std::cell::Cell;
use std::rc::Rc;

use proptest::prelude::*;
use proptest::sample::Index;
use wb_core::{
    Aabb, CodecError, EncodedPoints, Lamport, Raw, Stroke, StrokeCodec, StrokeId, StrokePoints,
    WorldPos,
};

use super::config::{AUTHOR, BUDGET, COLOR, JUNK_LEN, MAX_POINTS, MAX_STEPS, POOL, SPAN, WIDTH};

/// `Raw`, counting its decodes; clones share the count.
#[derive(Debug, Clone, Default)]
pub struct CountingCodec {
    decodes: Rc<Cell<usize>>,
}

impl CountingCodec {
    pub fn decodes(&self) -> usize {
        self.decodes.get()
    }
}

impl StrokeCodec for CountingCodec {
    fn simplify(&self, raw: &[WorldPos]) -> Vec<WorldPos> {
        Raw.simplify(raw)
    }

    fn encode(&self, points: &[WorldPos], budget: usize) -> Result<Vec<EncodedPoints>, CodecError> {
        Raw.encode(points, budget)
    }

    fn decode(&self, encoded: &EncodedPoints) -> Result<StrokePoints, CodecError> {
        self.decodes.set(self.decodes.get() + 1);
        Raw.decode(encoded)
    }
}

/// Test strokes, numbered by author sequence.
pub trait StrokeExt: Sized {
    /// Stroke number `seq` holding `points` as they are, within `bbox`.
    fn numbered(seq: u64, points: EncodedPoints, bbox: Aabb) -> Self;
    /// Stroke number `seq` through `points`, encoded by `Raw` as one piece; `None` if they need
    /// more than one.
    fn through(seq: u64, points: &[WorldPos]) -> Option<Self>;
}

impl StrokeExt for Stroke {
    fn numbered(seq: u64, points: EncodedPoints, bbox: Aabb) -> Self {
        Stroke {
            id: StrokeId::new(AUTHOR, seq),
            lamport: Lamport(seq),
            color: COLOR,
            width: WIDTH,
            bbox,
            points,
        }
    }

    fn through(seq: u64, points: &[WorldPos]) -> Option<Self> {
        let [piece] = <[_; 1]>::try_from(Raw.encode(points, BUDGET).ok()?).ok()?;
        Some(Self::numbered(seq, piece, Aabb::around(points)))
    }
}

/// Boxes around test points.
pub trait AabbExt {
    /// The smallest box holding every point of `points`; the origin for none.
    fn around(points: &[WorldPos]) -> Self;
}

impl AabbExt for Aabb {
    fn around(points: &[WorldPos]) -> Self {
        let origin = WorldPos::new(0, 0);
        let Some((&first, rest)) = points.split_first() else {
            return Aabb::new(origin, origin);
        };
        rest.iter().fold(Aabb::new(first, first), |bbox, p| {
            Aabb::new(
                WorldPos::new(bbox.min.x.min(p.x), bbox.min.y.min(p.y)),
                WorldPos::new(bbox.max.x.max(p.x), bbox.max.y.max(p.y)),
            )
        })
    }
}

#[derive(Debug, Clone)]
pub enum Step {
    Get(Index),
    SetResident(Aabb),
    Forget(Index),
}

fn position() -> impl Strategy<Value = WorldPos> {
    (-SPAN..=SPAN, -SPAN..=SPAN).prop_map(|(x, y)| WorldPos::new(x, y))
}

fn area() -> impl Strategy<Value = Aabb> {
    (position(), position()).prop_map(|(a, b)| Aabb::around(&[a, b]))
}

/// Distinct strokes spread over the area sessions look at, mostly well formed; some hold
/// bytes that do not decode.
pub fn board() -> impl Strategy<Value = Vec<Stroke>> {
    let stroke = prop_oneof![
        9 => prop::collection::vec(position(), 0..MAX_POINTS)
            .prop_filter_map("fits one piece", |points| Stroke::through(0, &points)),
        1 => (prop::collection::vec(any::<u8>(), 0..JUNK_LEN), area())
            .prop_map(|(junk, bbox)| Stroke::numbered(0, EncodedPoints(junk), bbox)),
    ];
    prop::collection::vec(stroke, 1..=POOL).prop_map(|strokes| {
        (0..)
            .zip(strokes)
            .map(|(seq, stroke)| Stroke::numbered(seq, stroke.points, stroke.bbox))
            .collect()
    })
}

/// Gets outweigh the rest, so most strokes are asked for again after a forget or a move.
pub fn session() -> impl Strategy<Value = Vec<Step>> {
    let step = prop_oneof![
        4 => any::<Index>().prop_map(Step::Get),
        1 => area().prop_map(Step::SetResident),
        1 => any::<Index>().prop_map(Step::Forget),
    ];
    prop::collection::vec(step, 0..MAX_STEPS)
}
