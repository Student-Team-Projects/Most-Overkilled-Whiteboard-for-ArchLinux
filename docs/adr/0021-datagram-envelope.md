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
