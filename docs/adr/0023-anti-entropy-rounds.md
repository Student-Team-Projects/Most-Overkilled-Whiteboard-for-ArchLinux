# 0023 — Hash-first stateless anti-entropy rounds

Status: accepted · 2026-10-15 · closes G10

- **Decision:** every 1 s choose one random peer and run push-pull, comparing state hashes first.
  The board hash is the XOR of 128-bit hashes of all op ids (adds and erases), maintained in
  `Board` with O(1) updates. On mismatch, exchange stateless range pages: “my ids in `[lo, hi)` are
  …”; the receiver answers that page alone, pushing the ops whose ids the sender lacks and
  requesting ids it lacks. A lost page is retried next round; no round state is kept. Page budget is
  `max_datagram − envelope`.
- **Testing:** isolate hash updates, range splitting, page build/answer, envelope parsing and codec
  splitting with unit tests and proptests; the simulator is the last line.
- **Open:** G26 (#60, bulk catch-up after rejoin) and G27 (#61, range hashes when whole-board hashes
  rarely match during live drawing) may amend this approach.
