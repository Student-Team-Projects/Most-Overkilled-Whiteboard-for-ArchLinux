# 0026 — `Board` is a seam trait; `TombstoneBoard` is its layer-0 impl

Status: proposed · 2026-10-27 · T09 review (#98)

- **Context:** the `Board` contract was agreed as a concrete type with laws. In review of T09, the
  guarantees were hard to find in code: they lived in `docs/contracts/board.md`, while the type's
  rustdoc only pointed there. Every other piece of replicated or swappable logic is a seam.
- **Decision:** `Board` becomes a trait in `wb-core` with `draw`, `erase`, `apply`, `merge`,
  `visible`, `state_hash` and `erase_ops`, and `Default` as the empty board. Its rustdoc states
  every guarantee and non-guarantee of the contract in plain words. `TombstoneBoard` (strokes +
  erase ops + tombstones, ADR 0001) is the layer-0 impl and the oracle; it takes the state-hash
  item hasher as a type parameter, XXH3-128 by default. `erase_ops` is new: it exposes the
  retained erase ops of guarantee 7, which anti-entropy resends and the suite must observe.
- **Why:** one documented interface holds the replicated state's guarantees; a later impl (e.g.
  a faster index of visible strokes) can be swapped in and must pass the same generic
  conformance suite (`tests/conformance/board.rs`), as for every other seam.
- **Consequences:** callers name `TombstoneBoard` to build a board and use `Board` methods on it;
  `merge(&Self)` and `impl Iterator` returns make the trait unusable as `dyn Board`, which no
  caller needs. Amends ADR 0001 (where the state lives) and the agreed `board.md` contract, so it
  needs all 3 before merge.
