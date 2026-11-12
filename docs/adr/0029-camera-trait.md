# 0029 — `Camera` is a seam trait, one impl per canvas

Status: accepted · 2026-11-12 (all 3, after the M0 demo) · T11 review (#111)

- **Context:** `camera.md` agreed on `Camera` as one concrete type, an affine matrix. The app will
  have several canvases, and each moves its view differently:
  - one fitted page (T13): no pan or zoom, the page always fills the window;
  - a stack of A4 pages (MVP, ADR 0019, ADR 0027): scrolls vertically within the stack, zoom
    fitted to the page width;
  - an infinite canvas (M4): free pan and zoom in every direction.

  The T11 review asked for `Camera` as a trait, like every other seam.
- **Decision:** `Camera` is a seam trait in `wb-app`. `AffineCamera` is its first impl: plain
  `i64` world coordinates, offsets from its `i64` centre taken before any float math, and free pan
  and zoom. A canvas that limits the view, such as the page stack, gets its own impl and may wrap
  `AffineCamera` for the transform. The trait also builds a camera that fits a world area into a
  viewport (`fitting`), and converts lengths and areas, since tools and pages need them.
- **Why:**
  - Each canvas keeps its own rules about where the view may go, and drawing, erasing and
    rendering stay written once against the trait.
  - Every camera keeps the same four guarantees.
  - The infinite canvas comes as a new impl, not as edits to the page-stack camera.
- **Consequences:**
  - The contract's four guarantees bind every impl. The conformance suite runs on `AffineCamera`
    and becomes generic when the second impl lands.
  - `fitting` returns `Self`, so the trait is unusable as `dyn Camera`. No caller needs that:
    views and controllers are generic over the model's camera.
  - Amends the agreed `camera.md` contract, so it needs all 3.
