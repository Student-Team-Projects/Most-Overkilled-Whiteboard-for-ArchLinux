# 0028 — Zoom 1 is a fixed scale; a board opens fitted to one page

Status: proposed · 2026-11-02 · clarifies ADR 0014 · needs all 3

- **Context:** ADR 0014 says 1 world unit is 1/16 px at zoom 1, but not which pixel, and nothing
  says how a board first looks. ADR 0013 clamps zoom to [0.5, 2]. With pan and zoom (T28), the
  first view decides how much of the screen a page uses.
- **Decision:**
  - The "px" of ADR 0014 is a logical pixel: an egui point. At zoom 1 an A4 page is 595 × 842
    points on every screen, so a HiDPI screen does not halve it.
  - Zoom 1 is that fixed scale, not "fit the page".
  - A board opens at the zoom that fits one whole page into the canvas with 16 points of space
    around it, clamped to [0.5, 2], with page 0 at the top. Resizing the window keeps the zoom and
    the world point at the centre.
  - Pages are fixed A4. In the stacked view, panning stops when the view centre would leave the
    page stack: `x` within the page width, `y` from the top of page 0 to the bottom of the last.
- **Why:** the page fills the screen at the start, while zoom keeps one meaning for stroke widths
  and the chunk levels of ADR 0013.
- **Consequences:** in a very tall window the opening page is smaller than the window (zoom
  stops at 2) · seeing many pages at once needs a wider zoom range, which means changing ADR 0013.
