# 0031 — libp2p connects peers; our protocol runs on top

Status: accepted · 2026-11-16 · amends ADR 0003, 0004, 0025

- **Context:** the ADR 0025 path (STUN + Mainline + our own punching) has no relay, so peers behind
  symmetric NAT never connect, and traffic is unencrypted. rust-libp2p already has DHT, hole
  punching and a relay. Comparison and research: PR for this ADR, team HMMM's `virtual-network` RFC 0002/0003.
- **Decision:**
  - libp2p only **connects** peers. We take the subset we need (QUIC, Noise, identify, Kademlia,
    AutoNAT, Circuit Relay v2, DCUtR), behind a private adapter in `wb-net`. No libp2p type leaves it.
  - Direct P2P is the goal; the relay is the fallback when hole punching fails (symmetric NAT).
  - Board messages go over libp2p as **our** envelope (ADR 0021), length-prefixed on one stream
    protocol, at most `MAX_DATAGRAM` bytes each. One path for direct and relayed links.
  - Rendezvous: Kademlia on the public Amino DHT; a board's key is a hash of `"wb-board-v1"` + the
    `BoardId` bytes. Manual join dials `ip:port` as before.
  - Kacper hosts a bootstrap node and a relay on his homelab for the demo; public bootstrap nodes
    stay in the list, so his server is not the only way in.
  - Links are encrypted (Noise). Admission is unchanged: anyone with the session code joins
    (ADR 0025); checking the code → a separate grill after MVP.
- **Why:** relay fallback and encryption for free, and a protocol stack others maintain; our
  sans-IO core and tests stay as they are.
- **Consequences:**
  - ADR 0003: links are reliable streams, not UDP datagrams; anti-entropy still heals what a dropped
    link loses. `Transport` and `Sim` keep their contract, so `Broadcast`, `AntiEntropy` and the
    seeded chaos tests do not change.
  - ADR 0004: the net thread runs a Tokio runtime for the adapter; the rest stays threads + channels.
  - ADR 0025: STUN, Mainline and our punching are replaced; session code, manual `ip:port` and the
    error messages stay. #70, #71 and #75 need re-scoping.
  - A self-hosted relay is a server of ours, against ADR 0025's "no server of ours", accepted for the demo.
