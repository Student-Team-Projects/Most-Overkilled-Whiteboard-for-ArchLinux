# 0025 — Internet connect: candidates, STUN, DHT rendezvous

Status: accepted · 2026-10-16 · closes G29

- **Context:** peers sit behind home NATs; the stakeholder does not want to depend on one third party.
- **Decision:**
  - Every source yields **candidate** addresses: manual `ip:port`, DHT lookup (TURN relay later).
    The app punches all candidates in parallel for up to 60 s; the first direct reply wins. When a
    relay may be chosen over a direct path → G30. No strategy negotiation.
  - STUN (RFC 8489 Binding, via `stun_codec`) runs on the data socket, so the learned public address
    is the one peers punch. Any STUN server: a public one by default, `--stun host:port` or the modal.
  - Rendezvous is the BitTorrent Mainline DHT (`mainline`, no tokio), BEP 5: every peer announces its
    STUN port under info-hash = SHA-1 of `"wb-board-v1"` + the 8 `BoardId` bytes (big-endian) and
    reads the others with `get_peers`. Every 30 s: STUN again, re-announce, re-read. Session code =
    the 8 bytes in RFC 4648 base32, no padding, case-insensitive (13 chars); anyone with it can join.
  - Peer traffic and STUN share one UDP socket; the driver splits STUN (magic cookie, known transaction)
    from `WBRD` before envelope checks, so STUN never counts as invalid (ADR 0021).
    15 s idle → keepalive to each peer, from both sides. IPv4 only; `max_datagram` is one constant.
  - The app shows its own LAN and public (STUN) `ip:port`, so users can share it for manual join.
  - Two errors: no candidates ("no peers for this code — check the code, or the network may block
    the DHT") and candidates that never answer ("can't reach peer — probably NAT; relay is planned").
    TURN (hosted coturn or a public server, our client only) is post-MVP (G30).
- **Why:** standard protocols, no server of ours, and data always flows peer to peer; a peer whose
  address changes re-announces and is found again without anyone swapping codes.
- **Consequences:** amends ADR 0004 (DHT threads: `mainline`'s own worker plus one helper that calls its
  blocking API and sends candidates to the net thread by channel; the sim feeds candidates directly), ADR 0019 (DHT
  instead of a rendezvous server) and ADR 0016 (MVP `wb join` takes a code or `ip:port`; names stay
  M4). BEP 5 stores the announcer's source IP, so the DHT and data sockets must leave through the
  same route (VPN split routing and dual-WAN unsupported). DHT may be blocked on campus networks:
  manual `ip:port` is the fallback and works only when one side is reachable (same LAN, public IP,
  forwarded port); test the demo network early (T34). How it works → `docs/design/internet-connect.md`.
