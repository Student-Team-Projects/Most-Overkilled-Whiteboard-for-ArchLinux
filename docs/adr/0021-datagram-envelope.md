# 0021 — Versioned datagram envelope

Status: accepted · 2026-10-15 · closes G08

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
- **Amended 2026-11-16 (T26 #50, proposed — needs all 3):** drops are only counted on receive;
  log lines come out on the driver's `tick`, at most one per reason per 10 s, saying how many were
  dropped since the last line. The first line waits for the next tick instead of logging at once.
  Why: one output path, and no count is lost when the drops stop. Counters stay exact.
