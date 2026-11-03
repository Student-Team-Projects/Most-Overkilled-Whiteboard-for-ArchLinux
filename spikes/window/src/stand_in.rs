#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldPos {
    pub x: i64,
    pub y: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aabb {
    pub min: WorldPos,
    pub max: WorldPos,
}

impl Aabb {
    pub fn contains(self, point: WorldPos) -> bool {
        (self.min.x..=self.max.x).contains(&point.x) && (self.min.y..=self.max.y).contains(&point.y)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba(pub [u8; 4]);

#[derive(Clone, Debug)]
pub struct StandInStroke {
    pub color: Rgba,
    pub width_in_world_units: u32,
    pub points: Vec<WorldPos>,
}

#[derive(Debug, Default)]
pub struct StandInBoard {
    pub strokes: Vec<StandInStroke>,
    last_page_index: u16,
}

impl StandInBoard {
    pub fn last_page_index(&self) -> u16 {
        self.last_page_index
    }

    pub fn apply_add_page_idempotently(&mut self, page_index: u16) {
        self.last_page_index = self.last_page_index.max(page_index);
    }
}
