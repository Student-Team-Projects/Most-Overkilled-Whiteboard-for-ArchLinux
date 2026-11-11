# 0003 — UDP datagrams + anti-entropy for transport

Status: accepted · 2026-10-13

- **Context:** the virtual network (other team) behaves like a LAN; we want gossip and discovery and
  want chaos (loss, partitions) to be visible, not hidden by TCP.
- **Decision:** all peer traffic is UDP datagrams. Lost messages are healed by periodic anti-entropy
  (peers exchange digests of what they have, then send what is missing). No TCP, no QUIC.
- **Why:** with ADR 0001 duplicates and reordering are harmless, so reliability comes from
  anti-entropy instead of the transport. Fits gossip/discovery naturally; no async runtime needed.
- **Amended 2026-11-09 (T16):** the `Transport` seam addresses datagrams by `SocketAddr`, not
  `PeerId`, since a UDP datagram carries no peer id; drivers map between them. The simulated
  transport's clock belongs to the sim driver, so a simulated `recv` never waits.
