//! Stroke codecs: turn a stroke's world points into compact bytes and back.

mod raw;

pub use raw::Raw;

use crate::{EncodedPoints, StrokePoints, WorldPos};

/// Turns a stroke's world points into encoded pieces and back.
///
/// The author first calls [`simplify`](Self::simplify), then [`encode`](Self::encode)s its
/// result. A stroke too large for one piece is split into several; each fits the byte budget on
/// its own, and each piece after the first starts with the last point of the one before it.
///
/// Every codec keeps these guarantees:
///
/// - **Exact round trip after `simplify`.** When `encode` succeeds, decoding every piece,
///   expanding each to origin plus offset, and joining them while dropping the point each piece
///   repeats from the one before gives back exactly the points that were encoded.
/// - `simplify` keeps the first and the last point, and every raw point lies within the codec's
///   tolerance of the simplified path.
/// - `decode` never panics, whatever the bytes.
/// - A successful `encode` returns at least one piece.
///
/// Points that did not go through `simplify` may come back changed, and no compression ratio is
/// promised. Codecs are not interchangeable on the wire: peers must use the same one, so changing
/// the codec changes the protocol version.
///
/// See also: `docs/contracts/stroke-codec.md`.
pub trait StrokeCodec {
    /// Returns the lossy author-side reduction of `raw`: the first and last point are kept, and
    /// every raw point lies within the codec's tolerance of the result. Empty in, empty out.
    fn simplify(&self, raw: &[WorldPos]) -> Vec<WorldPos>;

    /// Encodes `points` into one or more pieces of at most `budget` bytes each.
    ///
    /// The stroke is split where the budget or an `i32` offset from a piece's origin requires it;
    /// the two pieces at a split share the point there.
    ///
    /// # Errors
    ///
    /// - [`CodecError::UnsplittableDelta`] if two consecutive points are further apart than an
    ///   `i32` offset reaches on some axis, whatever the budget: there is no point between them
    ///   to split at.
    /// - Otherwise [`CodecError::BudgetTooSmall`] if a piece cannot fit `budget`. Each codec
    ///   documents the budget from which this never happens.
    fn encode(&self, points: &[WorldPos], budget: usize) -> Result<Vec<EncodedPoints>, CodecError>;

    /// Decodes one piece. Never panics, whatever the bytes; the decoded offsets always expand to
    /// points without overflowing `i64`.
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
