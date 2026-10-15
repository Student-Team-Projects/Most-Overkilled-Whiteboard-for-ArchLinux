# 0020 — Split oversize strokes in the codec

Status: accepted · 2026-10-15 · closes G06

- **Context:** UDP datagrams have a fixed budget, and encoded offsets may not fit `i32`.
- **Decision:** split one stroke into pieces that share an endpoint. Piece budget = `max_datagram()`
  minus a complete Add packet with worst-case metadata and points-length prefix, measured once by serializing it
  (metadata is assigned after encoding, so the codec cannot measure the real packet).
  Offset overflow uses the same split path. Each piece is an independent stroke with its
  own OpId and Lamport. A concurrent stroke may be ordered between pieces at crossings (accepted);
  erasing removes only the touched pieces, which is better for long lines.
- **Why:** no fragmentation protocol is needed; joins reconstruct the exact point path.
- **Test:** proptest that joined pieces equal the input and every complete Add packet fits the datagram budget.
