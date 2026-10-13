# 0008 — Draw order = (Lamport timestamp, PeerId)

Status: accepted · 2026-10-13

- **Context:** the board is a set (ADR 0001), but overlapping strokes need one stacking order;
  arrival order differs between peers, so it would show different pictures.
- **Decision:** every stroke carries `lamport = max(lamport seen) + 1`; strokes are drawn sorted by
  `(lamport, PeerId)`.
- **Why:** same order on every peer; a stroke drawn after seeing another lies on top of it;
  concurrent strokes get an arbitrary but identical order.
