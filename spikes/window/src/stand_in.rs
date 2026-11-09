//! Stand-in copies of the `wb-core` types the spike needs. T13 swaps them for the real ones.

/// A point on the board. One unit is 1/16 px at zoom 1.
///
/// See also: ADR 0014.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldPos {
    /// Rightwards from the left edge of the pages.
    pub x: i64,
    /// Downwards from the top of the first page.
    pub y: i64,
}

/// An axis-aligned rectangle on the board, edges included.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aabb {
    /// Top-left corner: the smallest x and y inside.
    pub min: WorldPos,
    /// Bottom-right corner: the largest x and y inside.
    pub max: WorldPos,
}

impl Aabb {
    /// Whether `point` lies inside or on an edge.
    pub fn contains(self, point: WorldPos) -> bool {
        (self.min.x..=self.max.x).contains(&point.x) && (self.min.y..=self.max.y).contains(&point.y)
    }
}

/// A colour as red, green, blue and alpha bytes, not premultiplied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba(pub [u8; 4]);

/// A finished stroke. Stands in for `wb-core`'s `Stroke`, which keeps its points encoded.
#[derive(Clone, Debug)]
pub struct StandInStroke {
    /// The colour the whole stroke is drawn in.
    pub color: Rgba,
    /// Pen width, in world units, so it scales with zoom.
    ///
    /// See also: ADR 0013.
    pub width_in_world_units: u32,
    /// The pen positions in drawing order. One point is a dot; never empty.
    pub points: Vec<WorldPos>,
}

/// The strokes and pages of one board. Stands in for `wb-core`'s `Board`.
#[derive(Debug, Default)]
pub struct StandInBoard {
    /// Visible strokes, oldest first, so later ones paint on top.
    pub strokes: Vec<StandInStroke>,
    last_page_index: u32,
}

impl StandInBoard {
    /// Index of the last page; 0 when the board has one page.
    pub fn last_page_index(&self) -> u32 {
        self.last_page_index
    }

    /// Makes pages `0..=page_index` exist. Never removes pages, so two peers
    /// adding the same page at once get one new page.
    ///
    /// See also: draft ADR 0027.
    pub fn apply_add_page_idempotently(&mut self, page_index: u32) {
        self.last_page_index = self.last_page_index.max(page_index);
    }
}
