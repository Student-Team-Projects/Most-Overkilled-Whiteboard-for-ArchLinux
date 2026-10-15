# 0023 — Hash-first stateless anti-entropy rounds

Status: accepted · 2026-10-15 · closes G10

- **Decision:** every 1 s choose one random peer and run push-pull, comparing state hashes first.
  The hash covers canonical state: ids of all strokes ever added and the set of tombstoned target
  ids, not erase OpIds. Inserting a new stroke id or newly tombstoning a target updates the 128-bit
  hash in `Board` in O(1); equal canonical states have equal hashes. On mismatch, exchange stateless
  range pages of stroke ids and tombstone target ids in `[lo, hi)`. The receiver answers that page
  alone, identifying missing operations and requests. Each peer keeps a local cursor rotating
  through ranges, one range per round and wrapping, so every range is eventually compared; the
  cursor is not shared round state. A lost page is retried next round. Fill each page by serializing
  the candidate with its packet envelope and stopping before it exceeds `max_datagram()` (or reserve
  the worst-case postcard length-prefix and id sizes); postcard lengths and ids are variable-size.
  Do not hand-count an “empty Sync page” allowance.
- During join classification (`relation`), exchange summaries, range hashes, or id pages in
  comparison-only mode. Do not push or apply operations until the relation is known and, for
  `Diverged` or `Ahead`, the user chooses merge, copy, or discard. Version-vector fallback to range
  pages during join uses the same comparison-only mode.
- **Testing:** include a multi-page convergence test with a board requiring at least three pages.
- **Testing:** isolate hash updates, range splitting, page build/answer, envelope parsing and codec
  splitting with unit tests and proptests; the simulator is the last line.
- **Open:** G26 (#60, bulk catch-up after rejoin) and G27 (#61, range hashes when whole-board hashes
  rarely match during live drawing) may amend this approach.
