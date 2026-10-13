# 0010 — World coordinates are i32 fixed-point

Status: accepted · 2026-10-13

- **Context:** points must be bit-identical on every peer and compress well; the canvas is "infinite".
- **Decision:** `Point { x: i32, y: i32 }` in fixed world units; conversion to `f32` happens only
  relative to the camera, at render time.
- **Why:** exact equality across peers, small integer deltas for the stroke codec (ADR 0005), and no
  float precision loss far from the origin. Range ±2³¹ units is effectively unbounded.
