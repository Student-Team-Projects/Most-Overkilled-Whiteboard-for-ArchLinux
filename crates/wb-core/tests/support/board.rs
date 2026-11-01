//! Builders for ops and boards.

pub mod oracle;
pub mod strategy;

use wb_core::{Board, EncodedPoints, Lamport, Op, OpId, PeerId, Stroke};

use super::config::PEERS;

use super::config::{BBOX, COLOR, WIDTH};

/// Short op ids for tests.
pub trait OpIdExt {
    /// Id of op number `seq` by peer `peer`.
    fn of(peer: u64, seq: u64) -> Self;
    /// Id of generated universe op `i`: `(i % PEERS, i)`.
    fn universe(i: u64) -> Self;
}

impl OpIdExt for OpId {
    fn of(peer: u64, seq: u64) -> Self {
        OpId::new(PeerId(peer), seq)
    }

    fn universe(i: u64) -> Self {
        Self::of(i % PEERS, i)
    }
}

/// Ops with only the fields a `Board` reads.
pub trait OpExt {
    /// Add of a stroke with no points, with any id and Lamport.
    fn stroke(id: OpId, lamport: u64) -> Self;
    /// Erase op `id` of the stroke `target`.
    fn erasure(id: OpId, target: OpId) -> Self;
    /// The op's own id: the stroke's id for an Add.
    fn id(&self) -> OpId;
}

impl OpExt for Op {
    fn stroke(id: OpId, lamport: u64) -> Self {
        Op::Add(Stroke {
            id,
            lamport: Lamport(lamport),
            color: COLOR,
            width: WIDTH,
            bbox: BBOX,
            points: EncodedPoints(Vec::new()),
        })
    }

    fn erasure(id: OpId, target: OpId) -> Self {
        Op::Erase { id, target }
    }

    fn id(&self) -> OpId {
        match self {
            Op::Add(stroke) => stroke.id,
            Op::Erase { id, .. } => *id,
        }
    }
}

/// Shorthands for building and comparing boards.
pub trait BoardExt: Board {
    /// A board with `ops` applied in order.
    fn of(ops: &[Op]) -> Self;
    /// A new board with `self`, then `other`, merged into it.
    fn merged(&self, other: &Self) -> Self;
    /// The visible strokes, owned, so pictures compare with `==`.
    fn picture(&self) -> Vec<Stroke>;
    /// Draws a stroke with no points by `me`, as the app does: stamp, build, apply.
    fn draw_any(&mut self, me: PeerId) -> Stroke;
    /// Erases `target` by `me`, as the app does, and returns the erase op.
    fn erase(&mut self, me: PeerId, target: OpId) -> Op;
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
        let stamp = self.stamp(me);
        let op = Op::add(stamp, COLOR, WIDTH, EncodedPoints(Vec::new()), BBOX);
        self.apply(op.clone());
        match op {
            Op::Add(stroke) => stroke,
            Op::Erase { .. } => unreachable!("Op::add builds an Add"),
        }
    }

    fn erase(&mut self, me: PeerId, target: OpId) -> Op {
        let op = Op::erase(self.stamp(me), target);
        self.apply(op.clone());
        op
    }
}
