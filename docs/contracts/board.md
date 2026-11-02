# Contract: `Board`

Status: agreed 2026-10-20 · proposed change: pages (ADR 0026, not agreed yet) · The replicated board state (ADR 0001, 0008, 0026). A concrete type, not a trait — but the
laws below are its contract and its property tests. Format → `README.md` here.

## Operations

- `draw` — accepts the author peer, color, world-unit width, encoded points, and a bounding box that covers them; assigns identifiers and a Lamport timestamp, applies and returns the operation. Does not report errors.
- `erase` — accepts the author peer and target stroke; applies and returns the erase operation. Does not report errors.
- `add_page` — accepts the author peer and a page index; applies and returns the operation saying pages up to that index exist. Does not report errors.
- `apply` — accepts any operation and reports whether the visible picture changed. Does not report errors.
- `merge` — accepts another replica of the same board and unions its state. Does not report errors.
- `visible` — accepts no inputs and yields added, unerased strokes in draw order. Does not report errors.
- `last_page` — accepts no inputs and returns the index of the last existing page. Does not report errors.
- `state_hash` — accepts no inputs and returns the incrementally maintained 128-bit hash of canonical state: all stroke ids ever added, tombstoned target ids, and `last_page`.

## Guarantees

1. `merge` is commutative, associative and idempotent (as observed via `visible`).
2. `apply` is idempotent; applying a set of ops in any order gives the same `visible`.
3. Erase wins: once `Erase(target)` is applied, `target` is never visible, even if its `Add` arrives later.
4. `visible` is sorted by `(lamport, peer)` (ADR 0008).
5. `draw` gives a Lamport greater than every Lamport this board has seen; OpIds from one `me` never repeat.
6. Inserting a stroke id, newly tombstoning a target, or raising `last_page` updates `state_hash` in O(1); duplicate adds and repeated erases leave it unchanged. Erase OpIds are not hashed. Equal canonical state has equal hashes.
7. The board retains every erase op with its OpId and target, even before the target's Add arrives,
   and every add-page op with its OpId and page; anti-entropy resends them (`docs/contracts/anti-entropy.md`).
8. `last_page` is the maximum of every applied add-page index and of the page holding the lowest
   bounding-box point of every added stroke, erased ones included; 0 on an empty board. It never
   decreases, and replicas with the same ops agree on it, whatever the order (ADR 0026).

## Not guaranteed

- Decoding: `Board` never looks inside `EncodedPoints` (ADR 0014).
- Ordering between independently drawn split pieces: a concurrent stroke may be ordered between pieces at crossings (ADR 0020).
- Validation of foreign ops (no auth, README) · memory ever shrinking (tombstones stay, backlog P3).
- Removing pages: `last_page` never decreases, even when every stroke on the last page is erased (ADR 0026).
- Board mode: `Board` does not know whether the board is `Stacked` or `Infinite`; the mode is board metadata (ADR 0026).

## Conformance tests (proptest)

| Test | Checks |
|---|---|
| `merge_laws` on random boards | 1 |
| `apply_any_permutation` of a random op list | 2 |
| `erase_before_add` | 3 |
| `visible_is_sorted` | 4 |
| `draw_after_merge_is_on_top` | 5 |
| `state_hash_tracks_canonical_state_and_matches_equal_boards` (including two erases of one target) | 6 |
| `retains_erase_ops_with_op_id_and_target` (including erase before add) | 7 |
| `retains_add_page_ops_with_op_id_and_page` | 7 |
| `last_page_is_max_of_add_pages_and_stroke_pages` (any op order, concurrent adds of one page give one page) | 8 |
