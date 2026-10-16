# 0018 — Unpacked points live in a `StrokeCache` seam

Status: accepted · 2026-10-14

- **Context:** strokes stay packed (ADR 0014); without a cache the naive renderer would unpack every
  visible stroke every frame.
- **Decision:** a `StrokeCache` seam in `wb-app` holds unpacked points; renderer and eraser read
  through it. Layer 0 `DecodeAll` unpacks once on arrival and keeps everything; M4 (re-planned, ADR 0019)
  `ChunkWorkingSet` keeps only strokes in resident chunks + one ring of margin, under a byte budget.
- **Why:** packed stays the single truth and `Board` is unchanged; no per-frame unpacking from day 1;
  the fast cache is tested against `DecodeAll` like every other seam.
