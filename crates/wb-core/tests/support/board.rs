//! Builders for ops and boards, and a hasher that makes `state_hash` countable.

pub mod oracle;
pub mod strategy;

use wb_core::{Board, EncodedPoints, ItemHasher, Lamport, Op, OpId, PeerId, Stroke};

use super::config::{BBOX, COLOR, WIDTH};

/// Short op ids for tests.
pub trait OpIdExt {
    /// Id of op number `seq` by peer `peer`.
    fn of(peer: u64, seq: u64) -> Self;
}

impl OpIdExt for OpId {
    fn of(peer: u64, seq: u64) -> Self {
        OpId::new(PeerId(peer), seq)
    }
}

/// Ops with only the fields a `Board` reads.
pub trait OpExt {
    /// Add of a stroke with no points.
    fn add(id: OpId, lamport: u64) -> Self;
    /// Erase op `id` of the stroke `target`.
    fn erase(id: OpId, target: OpId) -> Self;
    /// The op's own id: the stroke's id for an Add.
    fn id(&self) -> OpId;
}

impl OpExt for Op {
    fn add(id: OpId, lamport: u64) -> Self {
        Op::Add(Stroke {
            id,
            lamport: Lamport(lamport),
            color: COLOR,
            width: WIDTH,
            bbox: BBOX,
            points: EncodedPoints(Vec::new()),
        })
    }

    fn erase(id: OpId, target: OpId) -> Self {
        Op::Erase { id, target }
    }

    fn id(&self) -> OpId {
        match self {
            Op::Add(stroke) => stroke.id,
            Op::Erase { id, .. } => *id,
        }
    }
}

/// Shorthands for building and comparing boards of any impl.
pub trait BoardExt: Board {
    /// A board with `ops` applied in order.
    fn of(ops: &[Op]) -> Self;
    /// A new board with `self`, then `other`, merged into it.
    fn merged(&self, other: &Self) -> Self;
    /// The visible strokes, owned, so pictures compare with `==`.
    fn picture(&self) -> Vec<Stroke>;
    /// Draws a stroke with no points by `me` and returns it.
    fn draw_any(&mut self, me: PeerId) -> Stroke;
}

impl<B: Board> BoardExt for B {
    fn of(ops: &[Op]) -> Self {
        let mut board = Self::default();
        for op in ops {
            board.apply(op.clone());
        }
        board
    }

    fn merged(&self, other: &Self) -> Self {
        let mut board = Self::default();
        board.merge(self);
        board.merge(other);
        board
    }

    fn picture(&self) -> Vec<Stroke> {
        self.visible().cloned().collect()
    }

    fn draw_any(&mut self, me: PeerId) -> Stroke {
        match self.draw(me, COLOR, WIDTH, EncodedPoints(Vec::new()), BBOX) {
            Op::Add(stroke) => stroke,
            Op::Erase { .. } => unreachable!("draw returns an Add"),
        }
    }
}

/// Hashes every item to 1, so a board's `state_hash` is the number of items it hashed.
pub struct CountingHasher;

impl ItemHasher for CountingHasher {
    fn hash128(_: &[u8]) -> u128 {
        1
    }
}
