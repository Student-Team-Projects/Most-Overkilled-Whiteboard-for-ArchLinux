# 0014 — i64 world coordinates; strokes stay encoded in memory

Status: accepted · 2026-10-14 · closes G03 · supersedes ADR 0010

- **Context:** the canvas is infinite, casts between integer widths cause bugs, and huge boards must
  fit in RAM.
- **Decision:**
  - 1 world unit = 1/16 px at zoom 1.0. Every coordinate and all coordinate math is `i64`.
  - A stroke is stored **encoded**: `i64` origin + bit-packed deltas (codec, ADR 0005) — in memory,
    on the wire and on disk.
  - It is decoded on demand (drawing, precise hit test) into `i64` origin + `i32` offsets.
- **Why:** no overflow and no cast mistakes; deltas are small, so a point costs bytes, not 16 B.
  Decoding only touches the strokes being drawn or hit-tested.
