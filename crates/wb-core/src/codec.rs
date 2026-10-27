//! The `StrokeCodec` trait and its codecs.

mod raw;

pub use raw::Raw;

use crate::{EncodedPoints, StrokePoints, WorldPos};

/// Turns a stroke's world points into byte pieces and back. Codecs are not interchangeable on
/// the wire: every peer must use the same one.
pub trait StrokeCodec {
    /// Returns a lossy reduction of `raw` to encode instead of it. Keeps the first and last
    /// point, and every raw point lies within the codec's tolerance of the result.
    fn simplify(&self, raw: &[WorldPos]) -> Vec<WorldPos>;

    /// Encodes `points` into at least one piece of at most `budget` bytes. Pieces split where the
    /// budget or an `i32` offset requires it, and each piece after the first starts with the last
    /// point of the one before. When `points` came from [`simplify`](Self::simplify), decoding the
    /// pieces and dropping those repeated points gives them back exactly; other input may change.
    ///
    /// # Errors
    ///
    /// - [`CodecError::UnsplittableDelta`] if two consecutive points are too far apart for an
    ///   `i32` offset.
    /// - [`CodecError::BudgetTooSmall`] if `budget` is too small for a piece.
    fn encode(&self, points: &[WorldPos], budget: usize) -> Result<Vec<EncodedPoints>, CodecError>;

    /// Decodes one piece. Never panics, whatever the bytes.
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
