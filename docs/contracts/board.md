# Contract: `Board`

Status: draft · The replicated board state (ADR 0001, 0008). A concrete type, not a trait — but the
laws below are its contract and its property tests. Format → `README.md` here.

## Signature

```rust
impl Board {
    /// Author a new stroke here: assigns OpId and Lamport, applies it, returns the op to send.
    pub fn draw(&mut self, me: PeerId, color: Rgba, width: u16, points: Vec<Point>) -> Op;
    /// Author an erase of `target` here; returns the op to send.
    pub fn erase(&mut self, me: PeerId, target: StrokeId) -> Op;
    /// Apply an op from anywhere. Returns whether the visible picture changed.
    pub fn apply(&mut self, op: Op) -> Changed;
    /// Union with another replica of the same board.
    pub fn merge(&mut self, other: &Board);
    /// Visible (added, not erased) strokes in draw order.
    pub fn visible(&self) -> impl Iterator<Item = &Stroke>;
}
```

## Guarantees

1. `merge` is commutative, associative and idempotent (as observed via `visible`).
2. `apply` is idempotent; applying a set of ops in any order gives the same `visible`.
3. Erase wins: once `Erase(target)` is applied, `target` is never visible, even if its `Add` arrives later.
4. `visible` is sorted by `(lamport, peer)` (ADR 0008).
5. `draw` gives a Lamport greater than every Lamport this board has seen; OpIds from one `me` never repeat.

## Not guaranteed

- Validation of foreign ops (no auth, README) · memory ever shrinking (tombstones stay, backlog P3).

## Conformance tests (proptest)

| Test | Checks |
|---|---|
| `merge_laws` on random boards | 1 |
| `apply_any_permutation` of a random op list | 2 |
| `erase_before_add` | 3 |
| `visible_is_sorted` | 4 |
| `draw_after_merge_is_on_top` | 5 |
