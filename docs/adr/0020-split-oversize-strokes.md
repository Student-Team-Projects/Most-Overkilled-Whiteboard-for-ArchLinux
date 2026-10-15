# 0020 — Split oversize strokes in the codec

Status: accepted · 2026-10-15 · closes G06

- **Context:** UDP datagrams have a fixed budget, and encoded offsets may not fit `i32`.
- **Decision:** `encode(points, budget) -> Result<Vec<EncodedPoints>, CodecError>` splits one stroke
  into one or more pieces. Every piece fits the budget and neighbouring pieces share one point;
  offset overflow uses the same split path. `budget = max_datagram − envelope − op header`.
  The caller calls `draw` once for each piece, so each piece is an independent stroke with its own
  `OpId` and Lamport timestamp. Another peer's concurrent stroke may be ordered between pieces at
  crossings; this rare case is accepted. The split pieces are several strokes under ADR 0002, and
  erasing removes only the touched piece(s), which is better for long lines.
- **Why:** no fragmentation protocol is needed; joins reconstruct the exact point path.
- **Test:** proptest that joined pieces equal the input and every piece fits the budget.
