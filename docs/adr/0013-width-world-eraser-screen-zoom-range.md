# 0013 — Width in world units, eraser in screen pixels, zoom 0.5×–2×

Status: accepted · 2026-10-14 · closes G02

- **Context:** widths must be cacheable in chunk tiles; the eraser must feel the same at any zoom.
- **Decision:** a stroke's width is the width the user selected, stored in world units — zooming in
  makes it look wider. The eraser radius is in screen pixels, converted at use. Zoom range is
  **[0.5×, 2×]**.
- **Why:** world widths keep chunk tiles valid per zoom level; a narrow zoom range keeps the chunk
  cache small (few levels).
