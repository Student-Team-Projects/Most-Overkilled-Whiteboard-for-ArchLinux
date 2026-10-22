//! The replicated board state (`docs/contracts/board.md`, ADR 0001, 0008).

use std::collections::{BTreeMap, BTreeSet};

use xxhash_rust::xxh3::Xxh3;

use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, StrokeId};

/// Domain tags so a stroke id and a tombstoned target with the same id hash differently.
const ADDED: u8 = 1;
const TOMBSTONED: u8 = 2;

/// One replica of a board: every stroke ever added plus every erase op (ADR 0001).
///
/// Ordered collections keep iteration deterministic, so the simulator replays exactly.
#[derive(Debug, Default)]
pub struct Board {
    /// Every stroke ever added, erased or not.
    strokes: BTreeMap<StrokeId, Stroke>,
    /// Added and unerased strokes in draw order `(lamport, peer, seq)` (ADR 0008).
    order: BTreeSet<(Lamport, StrokeId)>,
    /// Every erase op: its id → its target (guarantee 7).
    erases: BTreeMap<OpId, StrokeId>,
    /// Erased stroke ids, whether or not their Add has arrived (guarantee 3).
    tombstones: BTreeSet<StrokeId>,
    /// Next unused sequence number per peer seen (guarantee 5).
    next_seq: BTreeMap<PeerId, u64>,
    /// Highest Lamport timestamp seen (guarantee 5).
    max_lamport: u64,
    /// Wrapping sum of element hashes of the canonical state: order-independent and updated in
    /// O(1) per new element (guarantee 6).
    hash: u128,
}

impl Board {
    /// An empty board.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Draws a new stroke by `me` on top of everything seen, applies it and returns its Add op.
    ///
    /// The caller passes a `bbox` that covers `points`; the board never decodes them.
    /// Contract: `board.md` → `draw`, guarantee 5.
    #[must_use = "the op must be broadcast"]
    pub fn draw(
        &mut self,
        me: PeerId,
        color: Rgba,
        width: u32,
        points: EncodedPoints,
        bbox: Aabb,
    ) -> Op {
        let op = Op::Add(Stroke {
            id: self.next_op_id(me),
            // Saturates only on a foreign Lamport of `u64::MAX`; foreign ops are not validated.
            lamport: Lamport(self.max_lamport.saturating_add(1)),
            color,
            width,
            bbox,
            points,
        });
        self.apply(op.clone());
        op
    }

    /// Erases `target` by `me`, applies the erase and returns its op.
    ///
    /// Contract: `board.md` → `erase`, guarantees 3 and 5.
    #[must_use = "the op must be broadcast"]
    pub fn erase(&mut self, me: PeerId, target: StrokeId) -> Op {
        let op = Op::Erase {
            id: self.next_op_id(me),
            target,
        };
        self.apply(op.clone());
        op
    }

    /// Applies any op, local or remote, and returns whether the visible picture changed.
    ///
    /// Contract: `board.md` → `apply`, guarantees 2, 3, 6 and 7.
    pub fn apply(&mut self, op: Op) -> bool {
        match op {
            Op::Add(stroke) => self.add(stroke),
            Op::Erase { id, target } => self.tombstone(id, target),
        }
    }

    /// Unions `other` (a replica of the same board) into this one.
    ///
    /// Contract: `board.md` → `merge`, guarantee 1.
    pub fn merge(&mut self, other: &Board) {
        for (id, stroke) in &other.strokes {
            if !self.strokes.contains_key(id) {
                self.add(stroke.clone());
            }
        }
        for (&id, &target) in &other.erases {
            self.tombstone(id, target);
        }
    }

    /// Added and unerased strokes in draw order `(lamport, peer)`; `seq` breaks ties, which only
    /// unvalidated foreign ops can create.
    ///
    /// Contract: `board.md` → `visible`, guarantees 3 and 4.
    pub fn visible(&self) -> impl Iterator<Item = &Stroke> {
        self.order.iter().filter_map(|(_, id)| self.strokes.get(id))
    }

    /// 128-bit hash of the canonical state: stroke ids ever added plus tombstoned targets.
    ///
    /// Contract: `board.md` → `state_hash`, guarantee 6; `anti-entropy.md` guarantee 5.
    #[must_use]
    pub fn state_hash(&self) -> u128 {
        self.hash
    }

    fn add(&mut self, stroke: Stroke) -> bool {
        let id = stroke.id;
        self.observe(id);
        self.max_lamport = self.max_lamport.max(stroke.lamport.0);
        if self.strokes.contains_key(&id) {
            return false;
        }
        let key = (stroke.lamport, id);
        self.strokes.insert(id, stroke);
        self.hash = self.hash.wrapping_add(element_hash(ADDED, id));
        !self.tombstones.contains(&id) && self.order.insert(key)
    }

    fn tombstone(&mut self, id: OpId, target: StrokeId) -> bool {
        self.observe(id);
        // The target's id is taken even before its Add arrives.
        self.observe(target);
        self.erases.insert(id, target);
        if !self.tombstones.insert(target) {
            return false;
        }
        self.hash = self.hash.wrapping_add(element_hash(TOMBSTONED, target));
        self.strokes
            .get(&target)
            .is_some_and(|stroke| self.order.remove(&(stroke.lamport, target)))
    }

    /// Records `id` as used, so `next_op_id` never hands it out again.
    fn observe(&mut self, id: OpId) {
        let next = self.next_seq.entry(id.peer).or_default();
        // Saturates only on a foreign `seq` of `u64::MAX`, like the Lamport in `draw`.
        *next = (*next).max(id.seq.saturating_add(1));
    }

    fn next_op_id(&self, me: PeerId) -> OpId {
        OpId {
            peer: me,
            seq: self.next_seq.get(&me).copied().unwrap_or_default(),
        }
    }
}

/// 128-bit hash of one canonical-state element.
///
/// xxh3 is a specified algorithm, so every peer and build computes the same value; std's hashers
/// promise neither that nor 128 bits. A rare collision is healed by anti-entropy range checks
/// (`anti-entropy.md` guarantee 1).
fn element_hash(tag: u8, id: OpId) -> u128 {
    let mut hasher = Xxh3::new();
    hasher.update(&[tag]);
    hasher.update(&id.peer.0.to_le_bytes());
    hasher.update(&id.seq.to_le_bytes());
    hasher.digest128()
}

#[cfg(test)]
mod tests {
    //! Conformance tests for `docs/contracts/board.md`; each test names the guarantee it checks.

    use std::collections::{BTreeMap, BTreeSet};

    use proptest::prelude::*;

    use super::{ADDED, Board, TOMBSTONED, element_hash};
    use crate::{Aabb, EncodedPoints, Lamport, Op, OpId, PeerId, Rgba, Stroke, WorldPos};

    /// Most ops a generated universe holds.
    const UNIVERSE: usize = 40;

    const BBOX: Aabb = Aabb {
        min: WorldPos { x: 0, y: 0 },
        max: WorldPos { x: 0, y: 0 },
    };

    /// Op `i` of a generated universe; ids are unique and spread over 3 peers.
    fn op_id(i: u64) -> OpId {
        OpId {
            peer: PeerId(i % 3),
            seq: i,
        }
    }

    fn stroke(id: OpId, lamport: u64) -> Stroke {
        Stroke {
            id,
            lamport: Lamport(lamport),
            color: Rgba([0, 0, 0, 255]),
            width: 16,
            bbox: BBOX,
            points: EncodedPoints(Vec::new()),
        }
    }

    fn draw(board: &mut Board, me: PeerId) -> Stroke {
        let op = board.draw(
            me,
            Rgba([255, 0, 0, 255]),
            32,
            EncodedPoints(vec![1, 2]),
            BBOX,
        );
        let Op::Add(stroke) = op else {
            panic!("draw returns an Add")
        };
        stroke
    }

    /// A universe of ops with unique ids. Lamports collide on purpose; an erase targets any op id
    /// of the universe (an Add, an Erase, a later op) or one that never exists.
    fn ops() -> impl Strategy<Value = Vec<Op>> {
        prop::collection::vec((any::<bool>(), 0..4u64, 0..48u64), 0..UNIVERSE).prop_map(|specs| {
            (0u64..)
                .zip(specs)
                .map(|(i, (is_add, lamport, target))| {
                    if is_add {
                        Op::Add(stroke(op_id(i), lamport))
                    } else {
                        Op::Erase {
                            id: op_id(i),
                            target: op_id(target),
                        }
                    }
                })
                .collect()
        })
    }

    /// An op universe and the same ops shuffled.
    fn permutations() -> impl Strategy<Value = (Vec<Op>, Vec<Op>)> {
        ops().prop_flat_map(|ops| (Just(ops.clone()), Just(ops).prop_shuffle()))
    }

    /// The ops of three replicas, each a random subset of one universe.
    fn replicas() -> impl Strategy<Value = [Vec<Op>; 3]> {
        (ops(), prop::collection::vec(any::<u8>(), UNIVERSE)).prop_map(|(ops, masks)| {
            std::array::from_fn(|r| {
                ops.iter()
                    .zip(&masks)
                    .filter(|(_, mask)| *mask & (1 << r) != 0)
                    .map(|(op, _)| op.clone())
                    .collect()
            })
        })
    }

    fn board_of<'a>(ops: impl IntoIterator<Item = &'a Op>) -> Board {
        let mut board = Board::new();
        for op in ops {
            board.apply(op.clone());
        }
        board
    }

    /// An empty board with `a`, then `b`, merged into it.
    fn merged(a: &Board, b: &Board) -> Board {
        let mut board = Board::new();
        board.merge(a);
        board.merge(b);
        board
    }

    fn picture(board: &Board) -> Vec<Stroke> {
        board.visible().cloned().collect()
    }

    /// The expected `state_hash`, from the ops alone: the added ids plus the erase targets.
    fn canonical_hash<'a>(ops: impl IntoIterator<Item = &'a Op>) -> u128 {
        let (mut added, mut tombstoned) = (BTreeSet::new(), BTreeSet::new());
        for op in ops {
            match op {
                Op::Add(stroke) => added.insert(stroke.id),
                Op::Erase { target, .. } => tombstoned.insert(*target),
            };
        }
        let added = added.into_iter().map(|id| element_hash(ADDED, id));
        let tombstoned = tombstoned
            .into_iter()
            .map(|id| element_hash(TOMBSTONED, id));
        added.chain(tombstoned).fold(0, u128::wrapping_add)
    }

    proptest! {
        /// Guarantee 1, plus a merged board's `state_hash` (guarantee 6).
        #[test]
        fn merge_laws([a, b, c] in replicas()) {
            let (a_ops, b_ops) = (a.clone(), b.clone());
            let [a, b, c] = [a, b, c].map(|ops| board_of(&ops));
            let ab = merged(&a, &b);
            prop_assert_eq!(picture(&ab), picture(&merged(&b, &a)));
            prop_assert_eq!(
                picture(&merged(&ab, &c)),
                picture(&merged(&a, &merged(&b, &c)))
            );
            prop_assert_eq!(picture(&merged(&a, &a)), picture(&a));

            prop_assert_eq!(ab.state_hash(), merged(&b, &a).state_hash());
            prop_assert_eq!(ab.state_hash(), canonical_hash(a_ops.iter().chain(&b_ops)));
        }

        /// Guarantee 2; `apply` reports exactly when the picture changes.
        #[test]
        fn apply_any_permutation((ops, shuffled) in permutations()) {
            let mut board = Board::new();
            for op in &shuffled {
                let before = picture(&board);
                let changed = board.apply(op.clone());
                prop_assert_eq!(changed, picture(&board) != before);
            }
            let settled = picture(&board);
            prop_assert_eq!(&settled, &picture(&board_of(&ops)));
            for op in ops {
                prop_assert!(!board.apply(op));
            }
            prop_assert_eq!(picture(&board), settled);
        }

        /// Guarantees 3 and 4: `visible` equals the unerased adds, sorted by `(lamport, peer)`.
        #[test]
        fn visible_is_sorted(ops in ops()) {
            let erased: BTreeSet<OpId> = ops
                .iter()
                .filter_map(|op| match op {
                    Op::Erase { target, .. } => Some(*target),
                    Op::Add(_) => None,
                })
                .collect();
            let mut expected: Vec<Stroke> = ops
                .iter()
                .filter_map(|op| match op {
                    Op::Add(stroke) if !erased.contains(&stroke.id) => Some(stroke.clone()),
                    _ => None,
                })
                .collect();
            expected.sort_by_key(|stroke| (stroke.lamport, stroke.id));

            let board = board_of(&ops);
            prop_assert!(board.visible().map(|s| (s.lamport, s.id.peer)).is_sorted());
            prop_assert_eq!(picture(&board), expected);
        }

        /// Guarantee 5: above every Lamport seen, and no id seen as an op or an erase target.
        #[test]
        fn draw_after_merge_is_on_top([a, b, _] in replicas()) {
            let seen = || a.iter().chain(&b);
            let seen_lamport = seen()
                .filter_map(|op| match op {
                    Op::Add(stroke) => Some(stroke.lamport),
                    Op::Erase { .. } => None,
                })
                .max();
            let seen_ids: BTreeSet<OpId> = seen()
                .flat_map(|op| match op {
                    Op::Add(stroke) => vec![stroke.id],
                    Op::Erase { id, target } => vec![*id, *target],
                })
                .collect();

            let mut board = merged(&board_of(&a), &board_of(&b));
            let me = PeerId(1);
            let first = draw(&mut board, me);
            prop_assert!(Some(first.lamport) > seen_lamport);
            prop_assert_eq!(board.visible().last(), Some(&first));

            let Op::Erase { id: erase_id, .. } = board.erase(me, first.id) else {
                panic!("erase returns an Erase")
            };
            let second = draw(&mut board, me);
            prop_assert!(second.lamport > first.lamport);
            let mine = [first.id, erase_id, second.id];
            prop_assert_eq!(mine.iter().collect::<BTreeSet<_>>().len(), mine.len());
            prop_assert!(mine.iter().all(|id| !seen_ids.contains(id)));
        }

        /// Guarantee 6.
        #[test]
        fn state_hash_tracks_canonical_state_and_matches_equal_boards(
            (ops, shuffled) in permutations(),
        ) {
            let mut board = board_of(&ops);
            let hash = board.state_hash();
            prop_assert_eq!(hash, canonical_hash(&ops));
            prop_assert_eq!(hash, board_of(&shuffled).state_hash());

            // Duplicate adds, and a second erase of every target by a new erase op id, change
            // nothing.
            let mut log = ops.clone();
            for op in &ops {
                match op {
                    Op::Add(_) => prop_assert!(!board.apply(op.clone())),
                    Op::Erase { target, .. } => log.push(board.erase(PeerId(7), *target)),
                }
            }
            prop_assert_eq!(board.state_hash(), hash);

            // A new stroke id, then a new tombstone, each change it.
            let fresh = draw(&mut board, PeerId(7));
            let with_stroke = board.state_hash();
            prop_assert_ne!(with_stroke, hash);
            log.push(board.erase(PeerId(7), fresh.id));
            log.push(Op::Add(fresh));
            prop_assert_ne!(board.state_hash(), with_stroke);
            prop_assert_eq!(board.state_hash(), canonical_hash(&log));
        }

        /// Guarantee 7.
        #[test]
        fn retains_erase_ops_with_op_id_and_target(ops in ops()) {
            let expected: BTreeMap<OpId, OpId> = ops
                .iter()
                .filter_map(|op| match op {
                    Op::Erase { id, target } => Some((*id, *target)),
                    Op::Add(_) => None,
                })
                .collect();
            prop_assert_eq!(&board_of(&ops).erases, &expected);
        }
    }

    /// Guarantee 3: an erase that arrives before its Add, directly or through `merge`.
    #[test]
    fn erase_before_add() {
        let add = Op::Add(stroke(op_id(0), 1));
        let erase = Op::Erase {
            id: op_id(1),
            target: op_id(0),
        };

        let mut board = Board::new();
        assert!(!board.apply(erase.clone()));
        assert!(!board.apply(add.clone()));
        assert_eq!(board.visible().count(), 0);

        let only_erase = board_of([&erase]);
        let only_add = board_of([&add]);
        assert_eq!(merged(&only_erase, &only_add).visible().count(), 0);
        assert_eq!(merged(&only_add, &only_erase).visible().count(), 0);
    }

    /// Guarantee 5: an erase target whose Add has not arrived is an id `draw` must not reuse, or
    /// the new stroke would be born erased.
    #[test]
    fn draw_skips_ids_of_erase_targets() {
        let me = PeerId(1);
        let target = OpId { peer: me, seq: 2 };
        let mut board = board_of([&Op::Erase {
            id: OpId {
                peer: PeerId(0),
                seq: 0,
            },
            target,
        }]);
        for _ in 0..3 {
            let stroke = draw(&mut board, me);
            assert_ne!(stroke.id, target);
            assert_eq!(board.visible().last(), Some(&stroke));
        }
    }

    /// Foreign ops are not validated: counters already at `u64::MAX` saturate instead of panicking.
    #[test]
    fn saturated_foreign_counters_do_not_panic() {
        let me = PeerId(1);
        let mut board = board_of([&Op::Add(stroke(
            OpId {
                peer: me,
                seq: u64::MAX,
            },
            u64::MAX,
        ))]);
        let _ = draw(&mut board, me);
        let _ = board.erase(me, op_id(0));
    }

    /// Guarantee 7: an erase whose target never arrived is kept with its op id and survives
    /// `merge`.
    #[test]
    fn retains_erase_op_before_add() {
        let erase = Op::Erase {
            id: op_id(4),
            target: op_id(9),
        };
        let board = merged(&Board::new(), &board_of([&erase]));
        assert_eq!(board.erases, BTreeMap::from([(op_id(4), op_id(9))]));
    }

    /// Guarantee 6: erase op ids are not hashed; an added id and a tombstoned target hash apart.
    #[test]
    fn state_hash_ignores_erase_op_ids() {
        let target = op_id(0);
        let by_one = board_of([&Op::Erase {
            id: op_id(1),
            target,
        }]);
        let by_other = board_of([&Op::Erase {
            id: op_id(2),
            target,
        }]);
        assert_eq!(by_one.state_hash(), by_other.state_hash());

        let added = board_of([&Op::Add(stroke(target, 1))]);
        assert_ne!(added.state_hash(), by_one.state_hash());
    }
}
