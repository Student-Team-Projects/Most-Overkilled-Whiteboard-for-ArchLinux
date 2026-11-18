# 0021 — Versioned datagram envelope

Status: accepted · 2026-10-15 · closes G08

Amended by ADR 0032 (2026-11-16): the envelope is a message on a length-prefixed libp2p stream;
the envelope itself is capped at 1200 B. Oversize frames reset the stream; invalid envelopes
within the cap are dropped and counted while the stream stays up.

- **Decision:** packets are `Packet { magic: b"WBRD", version: u8, body: Body }`; `Body` is
  `Discovery(DiscoveryMsg) | Board { board: BoardId, msg: BoardMsg }`, and `BoardMsg` is
  `Op(Op) | Sync(SyncMsg)`. Discovery is board-agnostic. Bodies use postcard enums. Check magic
  and version before postcard decodes the body. Start at version 1; every wire change (including
  the stroke codec) bumps it. Support one version, with no negotiation.
- **Invalid packets:** drop and count wrong magic, unknown version, undecodable body, and foreign
  `BoardId` separately. Log errors immediately on the first of each reason, then at most once per
  10 s with “+N more”; counters remain exact. Version mismatch logs both versions loudly, usually
  indicating different builds.
- **Conformance tests:** with injected time, cover one exact counter per reason (all four), immediate
  first log per reason, per-reason 10 s throttling with correct “+N more” across several intervals,
  and a version-mismatch log containing both received and supported versions.
