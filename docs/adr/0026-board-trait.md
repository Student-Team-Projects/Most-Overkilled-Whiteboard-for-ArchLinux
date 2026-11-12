# 0026 — `Board` is a seam trait

Status: accepted · 2026-11-12 (all 3, after the M0 demo) · T09 review (#98)

- **Context:** in review of T09, the reviewer asked for `Board` as a trait, like every other seam,
  with one write path for all ops.
- **Decision:** `Board` is a seam trait in `wb-core`. `TombstoneBoard` (ADR 0001) is its layer-0
  impl and the oracle; no faster impl is planned yet. `draw`/`erase` are replaced by `stamp`, which
  reserves a fresh `OpId` + Lamport; the app builds the op and applies it like a remote one. The
  state hash stays one fixed algorithm, since peers compare it (protocol), not a parameter of the
  trait or the impl.
- **Why:** the board's promises sit in one documented interface, and any later impl must pass the
  same suite. One write path keeps the board about state only; broadcast is the app's job.
- **Consequences:** the generic conformance suite runs on every impl. `merge(&Self)` and the
  `impl Iterator` return of `visible` make it unusable as `dyn Board`, which no caller needs.
  The trait has no way to read retained erase ops yet, so guarantee 7 is tested on
  `TombstoneBoard` only; anti-entropy will add that read with its own ADR.
  Amends the agreed `board.md` contract, so it needs all 3.
