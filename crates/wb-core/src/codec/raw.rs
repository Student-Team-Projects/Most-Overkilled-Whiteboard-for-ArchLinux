//! The simplest codec: an origin plus plain `i32` offsets, written with postcard.

use postcard::ser_flavors::Size;
use serde::Serialize;

use super::{CodecError, StrokeCodec};
use crate::{EncodedPoints, StrokePoints, WorldPos};

/// The reference codec, which faster codecs are tested against: `simplify` returns the points
/// unchanged (tolerance 0), and a piece is its first point as the origin plus every point's
/// offset from it, written with postcard. The empty stroke is one piece with origin `(0, 0)`
/// and no offsets.
///
/// # Minimum budget
///
/// Use a budget of at least 33 bytes, the largest two-point piece; real budgets are about one
/// datagram. From there `encode` never returns `BudgetTooSmall`. Below it, the greedy split may
/// return it in edge cases where a different split would still fit.
#[derive(Debug, Clone, Copy)]
pub struct Raw;

impl StrokeCodec for Raw {
    fn simplify(&self, raw: &[WorldPos]) -> Vec<WorldPos> {
        raw.to_vec()
    }

    fn encode(&self, points: &[WorldPos], budget: usize) -> Result<Vec<EncodedPoints>, CodecError> {
        if points
            .windows(2)
            .any(|pair| offset(pair[0], pair[1]).is_none())
        {
            return Err(CodecError::UnsplittableDelta);
        }
        let Some((&first, rest)) = points.split_first() else {
            // An empty stroke is still one piece, with no points, so it round-trips to nothing.
            let empty = Piece::new(WorldPos::new(0, 0)).finish()?;
            return if empty.0.len() <= budget {
                Ok(vec![empty])
            } else {
                Err(CodecError::BudgetTooSmall)
            };
        };

        let mut pieces = Vec::new();
        let mut piece = Piece::start(first, budget)?;
        let mut prev = first;
        for &point in rest {
            if !piece.try_push(point, budget)? {
                // Split at the previous point, which both pieces share. Consecutive offsets fit
                // `i32`, so only the budget can stop `[prev, point]`.
                pieces.push(piece.finish()?);
                piece = Piece::start(prev, budget)?;
                if !piece.try_push(point, budget)? {
                    return Err(CodecError::BudgetTooSmall);
                }
            }
            prev = point;
        }
        pieces.push(piece.finish()?);
        Ok(pieces)
    }

    /// Also rejects trailing bytes, and offsets that overflow `i64` from the origin: bytes may
    /// come from any peer, and decoded points must expand without overflow.
    fn decode(&self, encoded: &EncodedPoints) -> Result<StrokePoints, CodecError> {
        let (points, rest): (StrokePoints, _) =
            postcard::take_from_bytes(&encoded.0).map_err(|_| CodecError::Malformed)?;
        let origin = points.origin;
        let expands = points.offsets.iter().all(|&[dx, dy]| {
            origin.x.checked_add(dx.into()).is_some() && origin.y.checked_add(dy.into()).is_some()
        });
        if rest.is_empty() && expands {
            Ok(points)
        } else {
            Err(CodecError::Malformed)
        }
    }
}

/// A piece being filled; tracks its serialized size as points are pushed.
struct Piece {
    points: StrokePoints,
    /// Serialized bytes of `points.offsets`, without their length prefix.
    offsets_size: usize,
}

impl Piece {
    fn new(origin: WorldPos) -> Self {
        Self {
            points: StrokePoints {
                origin,
                offsets: Vec::new(),
            },
            offsets_size: 0,
        }
    }

    /// A piece holding just `origin`, or `BudgetTooSmall`.
    fn start(origin: WorldPos, budget: usize) -> Result<Self, CodecError> {
        let mut piece = Self::new(origin);
        if piece.try_push(origin, budget)? {
            Ok(piece)
        } else {
            Err(CodecError::BudgetTooSmall)
        }
    }

    /// Adds `point` if its offset from the origin fits `i32` and the piece still fits `budget`.
    fn try_push(&mut self, point: WorldPos, budget: usize) -> Result<bool, CodecError> {
        let Some(offset) = offset(self.points.origin, point) else {
            return Ok(false);
        };
        let offsets_size = self.offsets_size + serialized_size(&offset)?;
        if self.size_with(self.points.offsets.len() + 1, offsets_size)? > budget {
            return Ok(false);
        }
        self.points.offsets.push(offset);
        self.offsets_size = offsets_size;
        Ok(true)
    }

    /// Serialized size with `count` offsets of `offsets_size` bytes: origin, length, offsets.
    fn size_with(&self, count: usize, offsets_size: usize) -> Result<usize, CodecError> {
        Ok(serialized_size(&self.points.origin)? + serialized_size(&count)? + offsets_size)
    }

    fn finish(self) -> Result<EncodedPoints, CodecError> {
        postcard::to_allocvec(&self.points)
            .map(EncodedPoints)
            .map_err(|_| CodecError::Malformed)
    }
}

/// Offset of `to` from `from`, if it fits `i32` on both axes.
fn offset(from: WorldPos, to: WorldPos) -> Option<[i32; 2]> {
    let axis = |a: i64, b: i64| b.checked_sub(a).and_then(|d| i32::try_from(d).ok());
    Some([axis(from.x, to.x)?, axis(from.y, to.y)?])
}

/// Bytes postcard writes for `value`; a sequence's length prefix is written like a `usize`.
///
/// Sizing and writing these plain types into a `Vec` cannot fail; were it to, `encode` would
/// report `Malformed` rather than panic.
fn serialized_size(value: &impl Serialize) -> Result<usize, CodecError> {
    postcard::serialize_with_flavor(value, Size::default()).map_err(|_| CodecError::Malformed)
}
