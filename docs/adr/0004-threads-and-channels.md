# 0004 — std threads + channels, no async runtime

Status: accepted · 2026-10-13

- **Context:** GUI needs the main thread; network needs a blocking UDP socket and timers.
- **Decision:** GUI on the main thread, one network thread, channels between them. No tokio.
  Exception: the DHT threads (ADR 0025).
- **Why:** light, no async colouring of APIs, and the protocol logic stays sans-IO (pure state
  machine), so the simulator can drive it without threads at all.

Amended by ADR 0031: libp2p connects peers and carries our envelope on a stream. Amended by ADR
0032 (2026-11-16): the net thread owns a Tokio runtime and swarm; the protocol core remains sans-IO.
