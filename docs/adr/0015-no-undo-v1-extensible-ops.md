# 0015 — No undo/redo in v1; the op set stays extensible

Status: accepted · 2026-10-14 · closes G04

- **Context:** undo in a CRDT can only add ops (erase, or re-add copies); it is not needed for v1.
- **Decision:** no undo/redo in v1. New op kinds must stay possible: `Op` is an enum that grows by
  new variants; each new variant must keep the `Board` laws (commutative, idempotent merge) and
  ships with a protocol version bump (G08).
- **Why:** keeps v1 small without closing the door on undo, redo or new tools later.
