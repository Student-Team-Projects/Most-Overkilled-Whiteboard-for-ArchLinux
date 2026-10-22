//! How stroke points are shrunk and put into bytes (`docs/contracts/stroke-codec.md`, ADR 0005).

mod raw;

#[cfg(test)]
mod conformance;

pub use raw::Raw;

use crate::{EncodedPoints, StrokePoints, WorldPos};

/// Turns world points into encoded pieces and back.
///
/// Codecs are not interchangeable on the wire: the codec is part of the protocol version.
pub trait StrokeCodec {
    /// Lossy author-side reduction of raw points; keeps the endpoints.
    ///
    /// Contract: `stroke-codec.md` → `simplify`, guarantee 2.
    fn simplify(&self, raw: &[WorldPos]) -> Vec<WorldPos>;

    /// Encodes `points` into pieces of at most `budget` bytes each; neighbouring pieces share an
    /// endpoint (ADR 0020).
    ///
    /// Contract: `stroke-codec.md` → `encode`, guarantees 1, 4 and 5.
    ///
    /// # Errors
    ///
    /// [`CodecError::BudgetTooSmall`] if a piece cannot fit `budget`,
    /// [`CodecError::UnsplittableDelta`] if consecutive points are too far apart, and
    /// [`CodecError::NoPoints`] for an empty input.
    fn encode(&self, points: &[WorldPos], budget: usize) -> Result<Vec<EncodedPoints>, CodecError>;

    /// Decodes one piece.
    ///
    /// Contract: `stroke-codec.md` → `decode`, guarantee 3.
    ///
    /// # Errors
    ///
    /// [`CodecError::Postcard`] if `encoded` is not a piece this codec wrote.
    fn decode(&self, encoded: &EncodedPoints) -> Result<StrokePoints, CodecError>;
}

/// Why a stroke could not be encoded or decoded.
#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    /// There are no points to encode.
    #[error("no points to encode")]
    NoPoints,
    /// A piece holding the next point does not fit the byte budget.
    #[error("a piece does not fit the budget of {0} bytes")]
    BudgetTooSmall(usize),
    /// Two consecutive points are further apart than an `i32` offset can reach.
    #[error("consecutive points are too far apart to encode")]
    UnsplittableDelta,
    /// postcard could not read the bytes as a piece, or could not write one.
    #[error("postcard: {0}")]
    Postcard(#[from] postcard::Error),
}
