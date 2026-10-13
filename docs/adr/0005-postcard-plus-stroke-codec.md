# 0005 — serde + postcard for messages and files; strokes through a swappable codec

Status: accepted · 2026-10-13

- **Context:** one format for UDP and save files; strokes dominate the bytes.
- **Decision:** messages and files are `serde` + `postcard` with a version byte. Stroke points go
  through a `StrokeCodec` seam: layer 0 = raw points, later = the team's existing pipeline
  (point reduction → binary delta → bit packing).
- **Why:** postcard is compact and simple; the codec is where the real size win is and can be
  benchmarked in isolation. Point reduction is lossy, so it runs once, at the author, before the
  stroke gets its id — every peer stores identical points.
