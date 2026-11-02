# 0024 — Three crates, split by what they hold

Status: accepted · 2026-10-16 · closes G31

- **Decision:** the workspace has 3 crates: `wb-core` (pure logic: model, CRDT, codec, index),
  `wb-net` (protocol, transports, discovery, sim + `wb-sim` binary) and `wb-app` (UI, rendering,
  PDF). Seams are module boundaries inside them. Replaces the 7 crates from T05.
- **Why:** fewer manifests and edges for a 3-person project; headless binaries still never link
  egui. Crates do not enforce "no sockets in core" anyway (`std::net` is std) — review does.
- **Amended 2026-10-22 (T08):** wire types (`Packet`, `Body`, `BoardMsg`) live in `wb-net`, since they
  hold its `SyncMsg` and `DiscoveryMsg`; `wb-core` keeps the domain types and a generic `Outgoing`.
