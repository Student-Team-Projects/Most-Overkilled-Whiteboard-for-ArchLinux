# Contract: `Board`

Status: agreed 2026-10-20 · The replicated board state (ADR 0001, 0008). A seam trait (ADR 0026); the laws
below are its contract and its property tests. Format → `README.md` here.

## Operations

- `stamp` — accepts the author peer; reserves and returns a fresh `OpId` and Lamport for one local op. The caller builds the op (`Op::add`, `Op::erase`) and applies it. Does not report errors.
- `apply` — accepts any operation, local or remote, and reports whether the visible picture changed. Does not report errors.
- `merge` — accepts another replica of the same board and unions its state. Does not report errors.
- `visible` — accepts no inputs and yields added, unerased strokes in draw order. Does not report errors.
- `state_hash` — accepts no inputs and returns the incrementally maintained 128-bit hash of canonical state: all stroke ids ever added plus tombstoned target ids.

## Guarantees

1. `merge` is commutative, associative and idempotent (as observed via `visible`).
2. `apply` is idempotent; applying a set of ops in any order gives the same `visible`.
3. Erase wins: once `Erase(target)` is applied, `target` is never visible, even if its `Add` arrives later.
4. `visible` is sorted by `(lamport, peer)` (ADR 0008).
5. `stamp` gives a Lamport greater than every Lamport this board has seen and an OpId it has never seen (as an op or an erase target); a stamp is never handed out twice, even if its op is never applied.
6. Inserting a stroke id or newly tombstoning a target updates `state_hash` in O(1); duplicate adds and repeated erases leave it unchanged. Erase OpIds are not hashed. Equal canonical state has equal hashes.
7. The board retains every erase op with its OpId and target, even before the target's Add arrives;
   anti-entropy resends them (`docs/contracts/anti-entropy.md`).

## Not guaranteed

- Decoding: `Board` never looks inside `EncodedPoints` (ADR 0014).
- Ordering between independently drawn split pieces: a concurrent stroke may be ordered between pieces at crossings (ADR 0020).
- Validation of foreign ops (no auth, README) · memory ever shrinking (tombstones stay, backlog P3).

## Impls

| Layer | Impl |
|---|---|
| 0 | `TombstoneBoard`: every stroke ever added, every erase op and a tombstone set — **the oracle** |

## Conformance tests (proptest)

| Test | Checks |
|---|---|
| `merge_laws` on random boards | 1 |
| `apply_any_permutation` of a random op list | 2 |
| `erase_before_add` | 3 |
| `visible_is_sorted` | 4 |
| `stamp_after_merge_is_on_top`, `stamp_skips_ids_of_erase_targets`, `unapplied_stamps_never_repeat` | 5 |
| `state_hash_tracks_canonical_state_and_matches_equal_boards` (including two erases of one target) | 6 |
| `retains_erase_ops_with_op_id_and_target` (including erase before add) | 7 |
