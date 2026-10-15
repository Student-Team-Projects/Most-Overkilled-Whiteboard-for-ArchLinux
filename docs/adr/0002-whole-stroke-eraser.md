# 0002 — Eraser deletes whole strokes

Status: accepted · 2026-10-13

- **Context:** a pixel eraser splits strokes into new geometry and new ids.
- **Decision:** the eraser tombstones every stroke it touches, nothing is split.
- **Why:** one tombstone per erase fits ADR 0001 directly. A pixel eraser stays a possible later
  layer (tombstone + new sub-strokes), not a v1 goal.

Amended by ADR 0020: a long line split into pieces is several strokes; the eraser removes the touched
piece(s).
