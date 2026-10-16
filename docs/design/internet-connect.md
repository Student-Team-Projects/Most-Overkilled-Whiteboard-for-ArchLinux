# Design: connecting peers over the internet

How two peers behind NATs find and reach each other, and where that fails.
Decision → ADR 0025 · seam rules → `docs/contracts/discovery.md` · join after connecting → `join-flow.md`.

```mermaid
sequenceDiagram
    participant A as Peer A
    participant S as any STUN server
    participant D as Mainline DHT
    participant B as Peer B
    A->>S: Binding request (data socket)
    S-->>A: your public ip:port
    A->>D: announce public port under hash(BoardId) · every 30 s: STUN + re-announce
    Note over A,B: A shares the code (full BoardId, base32) over chat
    B->>S: Binding request
    S-->>B: your public ip:port
    B->>D: announce · get_peers(hash(BoardId))
    A->>D: get_peers on its next round · finds B
    par punch all candidates, ≤ 60 s
        A->>B: punch
        B->>A: punch
    end
    A-->>B: board traffic, direct · keepalive after 15 s idle
```

Candidate rules and timers → ADR 0025. STUN and the DHT never carry board data; only a TURN relay
would (post-MVP, G30).

## What works and what fails

| Situation | Result | Fix |
|---|---|---|
| normal home NATs | ✅ punch works | — |
| same LAN | ✅ if the router hairpins | manual `ip:port`; LAN discovery (M4) |
| Wi-Fi switch, router reboot | ✅ usually heals on the next 30 s rounds | — |
| idle connection | ✅ kept open by keepalives | — |
| network blocks the DHT (campus, company) | ❌ nobody found | manual `ip:port` if one side is reachable |
| symmetric NAT, many mobile/CGNAT pairs | ❌ punch fails | TURN relay (G30) |
| network blocks all UDP | ❌ nothing connects | TURN over TCP (G30; needs an ADR 0003 exception) |

## Reading guide

| Rule | Source |
|---|---|
| candidates, STUN, DHT, timers, failure message | ADR 0025 |
| one socket, split by magic | ADR 0025, ADR 0021 |
| what `wb join` accepts | ADR 0025, ADR 0016 |
| relay fallback | open (G30) |
