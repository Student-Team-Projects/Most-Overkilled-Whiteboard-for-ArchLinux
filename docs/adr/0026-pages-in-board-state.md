# 0026 — Pages are board state; the board mode is fixed at creation

Status: proposed · 2026-11-02 · for G25 · needs all 3

- **Context:** pages exist only per instance, so a peer that joins or reopens a file can get
  strokes on pages it does not have, and an empty page added by one peer never reaches the others.
  T30 asks for an add-page that is idempotent across peers. The infinite canvas (M4) must not need
  a new board format.
- **Decision:**
  - New op `Op::AddPage { id: OpId, page: u32 }` meaning "pages `0..=page` exist". Adding a page at
    the end sends `page = last + 1`; two peers adding at once send the same `page` and get one new
    page, not two.
  - `Board::last_page()` = the maximum of every applied `AddPage.page` and of the page holding the
    lowest bounding-box point of every added stroke (erased ones too); 0 on an empty board;
    saturating at `u32::MAX`. The stroke part keeps a stroke whose `Add` overtakes its `AddPage`
    on an existing page.
  - Page `n` spans world `x` from 0 to 595 × 16 and `y` from `n` × (842 + 16) × 16 for 842 × 16
    units (A4 at 72 dpi, 16-unit gap, ADR 0014); its printable area is inset by 14 × 16 units.
  - `AddPage` ops are retained and resent like erase ops; `state_hash` covers `last_page()`.
  - The board mode, `Stacked` or `Infinite`, is board metadata fixed at creation, next to
    `BoardId` and the name: in the file header (G07) and in the join handshake. `Board` does not
    enforce it; in `Stacked` mode the author's input only accepts points inside a printable area.
    The MVP has only `Stacked`; in `Infinite` mode `last_page()` is unused.
- **Why:** a maximum over growing sets is a CRDT for free, so pages need no new merge machinery
  and survive join, reopen and reordering. A fixed mode needs no CRDT at all; switching modes
  later would need a last-writer-wins register, which v1 does not need.
- **Consequences:** protocol version bump (ADR 0015) · `anti-entropy.md` summaries and range pages
  must cover `AddPage` ops · the G07 file header gains the mode · pages are never removed: a page
  stays even when all its strokes are erased.
