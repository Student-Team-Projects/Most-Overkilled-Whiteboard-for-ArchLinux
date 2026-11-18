# 0028 — Stroke colours are opaque: no alpha

Status: proposed · 2026-11-02 · needs all 3 · recovered from the closed T27 spike (#104)

- **Context:** `Rgba` lets a stroke be see-through, but a see-through stroke is hard to draw
  right. A line is drawn as segments with a disc on every point, for round ends and joins without
  epaint's miter spikes; with alpha below 255 every overlap blends twice and shows as a darker dot.
  Drawing it right needs each stroke painted opaque to its own texture and composited, per stroke
  and per frame — a renderer we do not need for the MVP. The colour picker's additive mode has the
  same problem. Nothing else needs alpha: the eraser removes whole strokes (ADR 0002).
- **Decision:**
  - A stroke colour is red, green and blue only: `Rgba([u8; 4])` becomes `Rgb([u8; 3])` in
    `wb-core`, on the wire and in files.
  - The GUI offers only opaque colours: no alpha slider and no additive mode.
  - A highlighter or any see-through ink later needs its own ADR, together with the renderer
    that composites it.
- **Why:** simplicity. Opaque strokes render correctly with the plain approach, so there are no
  artifacts to explain, and one byte less per stroke.
- **Consequences:** protocol version bump (ADR 0015) · `Stroke` in `docs/contracts/README.md`
  names its colour as RGB · PDF export (T32) writes opaque colours only.
