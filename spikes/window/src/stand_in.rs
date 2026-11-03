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

#[derive(Clone, Debug)]
pub struct StandInStroke {
    pub unmultiplied_rgba: [u8; 4],
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
