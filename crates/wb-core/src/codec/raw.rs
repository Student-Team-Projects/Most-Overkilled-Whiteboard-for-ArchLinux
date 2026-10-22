//! Layer-0 codec: origin + plain `i32` offsets via postcard (`stroke-codec.md` → Impls).

use super::{CodecError, StrokeCodec};
use crate::{EncodedPoints, StrokePoints, WorldPos};

/// The oracle codec: no simplification; a piece is its first point as origin plus every point's
/// offset from it, serialized with postcard.
#[derive(Debug, Clone, Copy)]
pub struct Raw;

impl StrokeCodec for Raw {
    fn simplify(&self, raw: &[WorldPos]) -> Vec<WorldPos> {
        raw.to_vec()
    }

    fn encode(&self, points: &[WorldPos], budget: usize) -> Result<Vec<EncodedPoints>, CodecError> {
        let (&first, rest) = points.split_first().ok_or(CodecError::NoPoints)?;
        let mut pieces = Vec::new();
        let mut piece = Piece::start(first, budget)?;
        let mut prev = first;
        for &point in rest {
            if piece.push(point, budget).is_err() {
                // Split: the next piece starts at the previous point, shared by both (ADR 0020).
                // A piece that cannot take even one more point fails again with the reason.
                pieces.push(piece.finish()?);
                piece = Piece::start(prev, budget)?;
                piece.push(point, budget)?;
            }
            prev = point;
        }
        pieces.push(piece.finish()?);
        Ok(pieces)
    }

    fn decode(&self, encoded: &EncodedPoints) -> Result<StrokePoints, CodecError> {
        Ok(postcard::from_bytes(&encoded.0)?)
    }
}

/// A piece being filled, with its serialized size tracked as points are pushed.
struct Piece {
    points: StrokePoints,
    /// Serialized bytes of `points.offsets`, without their length prefix.
    offsets_size: usize,
}

impl Piece {
    fn start(origin: WorldPos, budget: usize) -> Result<Self, CodecError> {
        let mut piece = Self {
            points: StrokePoints {
                origin,
                offsets: Vec::new(),
            },
            offsets_size: 0,
        };
        piece.push(origin, budget)?;
        Ok(piece)
    }

    /// Adds `point` if its offset fits `i32` and the piece still fits `budget`; else leaves the
    /// piece unchanged.
    fn push(&mut self, point: WorldPos, budget: usize) -> Result<(), CodecError> {
        let origin = self.points.origin;
        let offset = [offset(origin.x, point.x), offset(origin.y, point.y)];
        let [Some(dx), Some(dy)] = offset else {
            return Err(CodecError::UnsplittableDelta);
        };
        let offsets_size =
            self.offsets_size + varint_size(zigzag(dx.into())) + varint_size(zigzag(dy.into()));
        let count = self.points.offsets.len() as u64 + 1;
        // postcard layout: origin x, y · offsets length · each offset's dx, dy — all varints.
        let size = varint_size(zigzag(origin.x))
            + varint_size(zigzag(origin.y))
            + varint_size(count)
            + offsets_size;
        if size > budget {
            return Err(CodecError::BudgetTooSmall(budget));
        }
        self.points.offsets.push([dx, dy]);
        self.offsets_size = offsets_size;
        Ok(())
    }

    fn finish(self) -> Result<EncodedPoints, CodecError> {
        Ok(EncodedPoints(postcard::to_allocvec(&self.points)?))
    }
}

/// `to - from` if it fits `i32`.
fn offset(from: i64, to: i64) -> Option<i32> {
    to.checked_sub(from).and_then(|d| i32::try_from(d).ok())
}

/// postcard's zigzag mapping of a signed integer: small magnitudes become small varints.
fn zigzag(v: i64) -> u64 {
    ((v << 1) ^ (v >> 63)).cast_unsigned()
}

/// Bytes of postcard's varint for `v`: 7 bits per byte.
fn varint_size(v: u64) -> usize {
    let bits = u64::BITS - (v | 1).leading_zeros();
    bits.div_ceil(7) as usize
}

#[cfg(test)]
mod tests;
