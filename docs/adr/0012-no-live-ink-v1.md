# 0012 — No live ink in v1: strokes appear on pen-up

Status: accepted · 2026-10-14 · closes G01

- **Context:** live ink (seeing a line grow while it is drawn) needs a second, non-CRDT message type.
- **Decision:** v1 sends a stroke only when the author lifts the pen. Live ink, if ever added, is a
  separate `LiveInk` seam whose messages never enter the `Board` and are never gossiped.
- **Why:** simplest, and lighter on the network.
