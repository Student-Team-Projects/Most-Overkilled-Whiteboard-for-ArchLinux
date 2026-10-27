# 0026 — `Board` is a seam trait

Status: proposed · 2026-10-27 · T09 review (#98)

- **Context:** in review of T09, the reviewer asked for `Board` as a trait, so its promises sit in
  one documented interface, like every other seam.
- **Decision:** `Board` is a seam trait in `wb-core`. `TombstoneBoard` (ADR 0001) is its layer-0
  impl and the oracle; no faster impl is planned yet. The state hash stays one fixed algorithm,
  since peers compare it (protocol), not a parameter of the trait or the impl.
- **Consequences:** the generic conformance suite runs on every impl. `merge(&Self)` and the
  `impl Iterator` return of `visible` make it unusable as `dyn Board`, which no caller needs.
  Amends the agreed `board.md` contract, so it needs all 3.
