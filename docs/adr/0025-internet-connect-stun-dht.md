# 0025 — Internet connect: candidates, STUN, DHT rendezvous

Status: accepted · 2026-10-16 · closes G29

- **Context:** peers sit behind home NATs; the stakeholder does not want to depend on one third party.
- **Decision:**
  - Every source yields **candidate** addresses: manual `ip:port`, LAN address, DHT lookup (TURN
    relay later). The app punches all candidates in parallel for up to 60 s; first reply wins, direct
    before relay. No strategy negotiation.
  - STUN (RFC 8489 Binding, via `stun_codec`) runs on the data socket, so the learned public address
    is the one peers punch. Any STUN server: a public one by default, `--stun host:port` or the modal.
  - Rendezvous is the BitTorrent Mainline DHT (`mainline`, no tokio, own socket): each peer publishes
    a signed record of its public + LAN address under a key derived from the `BoardId`, and re-publishes
    and re-reads every 30 s. Session code = the `BoardId` short form; anyone with it can join.
  - Peer traffic and STUN share one UDP socket, split by STUN magic cookie vs `WBRD` (ADR 0021).
    15 s idle → keepalive to each peer, from both sides. IPv4 only; `max_datagram` is one constant.
  - Punch timeout → "can't reach peer — probably NAT; relay is planned". TURN (hosted coturn or a
    public server, our client only) is post-MVP (G30).
- **Why:** standard protocols, no server of ours, and data always flows peer to peer; a peer whose
  address changes re-publishes and is found again without anyone swapping codes.
- **Consequences:** amends ADR 0019 (DHT instead of a rendezvous server) and ADR 0016 (MVP `wb join`
  takes a code or `ip:port`; names stay M4). DHT may be blocked on campus networks: manual `ip:port`
  is the fallback; test the demo network early (T34). How it works → `docs/design/internet-connect.md`.
