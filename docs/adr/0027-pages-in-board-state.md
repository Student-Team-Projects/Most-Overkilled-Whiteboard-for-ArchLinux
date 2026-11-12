# 0027 — Pages are board state; strokes stay on one page; vector PDF export

Status: accepted · 2026-11-12 (all 3, after the M0 demo) · closes G25 · first draft by Filip

- **Context:** the MVP is a vertical stack of A4 pages with PDF export of the pages that have
  something drawn (ADR 0019). If pages existed only per instance, a peer that joins or reopens a
  file could get strokes on pages it does not have, and an empty page added by one peer would never
  reach the others. T30 asks for an add-page that is idempotent across peers. The infinite canvas
  (M4) must not need a new board format.
- **Decision — pages:**
  - New op `Op::AddPage { id: OpId, count: u32 }` meaning "at least `count` pages exist". Adding a
    page at the end sends `count = page_count + 1`; two peers adding at once send the same `count`
    and get one new page, not two.
  - `Board::page_count()` = the maximum of 1, of every applied `AddPage.count`, and of one past the
    page holding the lowest bounding-box point of every added stroke (erased ones too); 1 on an
    empty board; saturating at `u32::MAX`. The stroke part keeps a stroke whose `Add` overtakes
    its `AddPage` on an existing page. It is on `Board` because only the board holds the `AddPage`
    ops; the page view, "add page" and the export dialog all need it.
  - Page `n` spans world `x` from 0 to 595 × 16 and `y` from `n` × (842 + 16) × 16 for 842 × 16
    units: A4 portrait at 72 dpi, one point = 16 world units (ADR 0014), and a 16 pt gap between
    pages. Its printable area is inset by 14 × 16 units.
  - `AddPage` ops are retained and resent like erase ops; `state_hash` covers `page_count()`.
  - The board mode, `Stacked` or `Infinite`, is board metadata fixed at creation, next to
    `BoardId` and the name: in the file header (G07) and in the join handshake. `Board` does not
    enforce it. The MVP has only `Stacked`; in `Infinite` mode `page_count()` is unused.
- **Decision — strokes on pages (`Stacked` mode):**
  - The author's input accepts points only inside a printable area. When the pointer leaves it,
    the stroke ends as if the pen were lifted; coming back onto a page starts a new stroke. So every
    stroke lies on one page and none crosses the gap.
  - The renderer and the PDF clip each page to its bounds, for the part of a line's width that
    overhangs the printable area.
- **Decision — PDF export:**
  - Vector, written with the `pdf-writer` crate: one PDF page of 595 × 842 pt per exported page;
    each visible stroke is a path in `visible()` order, its width in points = world width / 16,
    with round caps and joins, clipped to the page.
  - The export dialog lists every page with a checkbox; pages with something drawn are checked by
    default. A page has something drawn when a visible stroke's bounding box intersects it.
    Checked pages are written top to bottom.
- **Why:** a maximum over growing sets is a CRDT for free, so pages need no new merge machinery
  and survive join, reopen and reordering. A fixed mode needs no CRDT at all; switching modes
  later would need a last-writer-wins register, which v1 does not need. Ending a stroke at the
  page edge keeps the data honest (no line across the gap) and makes "the page of a stroke" exact.
  Vector PDF stays sharp at any zoom, maps exactly onto world units, and PDF draws round caps and
  joins itself; `pdf-writer` is small, has no dependencies and is maintained by the typst team.
- **Consequences:** protocol version bump (ADR 0015) · `anti-entropy.md` summaries and range pages
  must cover `AddPage` ops · the G07 file header gains the mode · pages are never removed: a page
  stays even when all its strokes are erased · two peers that add a page while apart (offline, a
  partition, a diverged join) get one page, so their strokes on it overlap; accepted, since live
  peers see each other's page within one round trip · T27's pen splits strokes at the page edge · T32
  gains the page checkboxes.
