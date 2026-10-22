//! How stroke points are shrunk and put into bytes (`docs/contracts/stroke-codec.md`, ADR 0005).

mod raw;

#[cfg(test)]
mod conformance;
#[cfg(test)]
mod testing;

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
    /// [`CodecError::UnsplittableDelta`] if two consecutive points are further apart than an
    /// `i32` offset reaches, else [`CodecError::BudgetTooSmall`] if a piece cannot fit `budget`.
    /// Each codec documents the minimum budget from which `BudgetTooSmall` never happens.
    fn encode(&self, points: &[WorldPos], budget: usize) -> Result<Vec<EncodedPoints>, CodecError>;

    /// Decodes one piece.
    ///
    /// Contract: `stroke-codec.md` → `decode`, guarantee 3.
    ///
    /// # Errors
    ///
    /// [`CodecError::Malformed`] if `encoded` is not a well-formed piece.
    fn decode(&self, encoded: &EncodedPoints) -> Result<StrokePoints, CodecError>;
}

/// Why a stroke could not be encoded or decoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum CodecError {
    /// Two consecutive points are further apart than an `i32` offset reaches.
    #[error("consecutive points are too far apart to encode")]
    UnsplittableDelta,
    /// A piece cannot fit the byte budget.
    #[error("a piece does not fit the byte budget")]
    BudgetTooSmall,
    /// The bytes are not a well-formed piece.
    #[error("encoded points are malformed")]
    Malformed,
}
