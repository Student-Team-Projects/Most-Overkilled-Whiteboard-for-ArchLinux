# 0025 — Internet connect: candidates, STUN, DHT rendezvous

Status: accepted · 2026-10-16 · closes G29

- **Context:** peers sit behind home NATs; the stakeholder does not want to depend on one third party.
- **Decision:**
  - Every source yields **candidate** addresses: manual `ip:port`, DHT lookup (TURN relay
    later). The app punches all candidates in parallel for up to 60 s; first reply wins, direct before
    relay. No strategy negotiation.
  - STUN (RFC 8489 Binding, via `stun_codec`) runs on the data socket, so the learned public address
    is the one peers punch. Any STUN server: a public one by default, `--stun host:port` or the modal.
  - Rendezvous is the BitTorrent Mainline DHT (`mainline`, no tokio, own socket), BEP 5: every peer
    announces its STUN port under info-hash = hash of the full `BoardId` and reads the others with
    `get_peers`. Every 30 s: STUN again, re-announce, re-read. Session code = the full `BoardId` in
    base32 (13 chars); anyone with it can join.
  - Peer traffic and STUN share one UDP socket; the driver splits STUN (magic cookie, known transaction)
    from `WBRD` before envelope checks, so STUN never counts as invalid (ADR 0021).
    15 s idle → keepalive to each peer, from both sides. IPv4 only; `max_datagram` is one constant.
  - Punch timeout → "can't reach peer — probably NAT; relay is planned". TURN (hosted coturn or a
    public server, our client only) is post-MVP (G30).
- **Why:** standard protocols, no server of ours, and data always flows peer to peer; a peer whose
  address changes re-announces and is found again without anyone swapping codes.
- **Consequences:** amends ADR 0004 (`mainline` runs its own worker thread behind a blocking API;
  the net thread calls it, the sim uses a fake DHT), ADR 0019 (DHT instead of a rendezvous server) and ADR 0016 (MVP `wb join`
  takes a code or `ip:port`; names stay M4). DHT may be blocked on campus networks: manual `ip:port`
  is the fallback and works only when one side is reachable (same LAN, public IP, forwarded port);
  test the demo network early (T34). Same-LAN peers without router hairpinning need manual `ip:port`
  until LAN discovery (M4). How it works → `docs/design/internet-connect.md`.
