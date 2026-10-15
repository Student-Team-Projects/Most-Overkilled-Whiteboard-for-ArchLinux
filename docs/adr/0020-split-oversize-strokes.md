# 0020 — Split oversize strokes in the codec

Status: accepted · 2026-10-15 · closes G06

- **Context:** UDP datagrams have a fixed budget, and encoded offsets may not fit `i32`.
- **Decision:** split one stroke into pieces that share an endpoint. A piece fits when its complete
  serialized Add packet is at most `max_datagram()`; size it by serializing, never by hand-counting.
  Offset overflow uses the same split path. Each piece is an independent stroke with its
  own OpId and Lamport. A concurrent stroke may be ordered between pieces at crossings (accepted);
  erasing removes only the touched pieces, which is better for long lines.
- **Why:** no fragmentation protocol is needed; joins reconstruct the exact point path.
- **Test:** proptest that joined pieces equal the input and every complete Add packet fits the datagram budget.
