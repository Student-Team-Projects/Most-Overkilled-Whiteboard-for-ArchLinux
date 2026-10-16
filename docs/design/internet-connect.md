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
    A->>D: publish {public, LAN} under key(BoardId), every 30 s
    Note over A,B: A shares the code (BoardId short form) over chat
    B->>S: Binding request
    S-->>B: your public ip:port
    B->>D: publish own record · read records for key(BoardId)
    A->>D: re-read (≤ 30 s) · finds B
    par punch all candidates, ≤ 60 s
        A->>B: punch
        B->>A: punch
    end
    A-->>B: board traffic, direct · keepalive after 15 s idle
```

Candidates come from manual `ip:port`, the LAN address and the DHT (later: a TURN relay). All are
tried at once; the first that answers is kept, direct before relay. STUN and the DHT never carry
board data; only a TURN relay would (post-MVP, G30).

## What works and what fails

| Situation | Result | Fix |
|---|---|---|
| normal home NATs | ✅ punch works | — |
| same LAN | ✅ via the LAN candidate | — |
| Wi-Fi switch, router reboot | ✅ heals in ~30–60 s (re-publish + re-read) | — |
| idle connection | ✅ kept open by keepalives | — |
| network blocks the DHT (campus, company) | ❌ nobody found | manual `ip:port` |
| symmetric NAT, many mobile/CGNAT pairs | ❌ punch fails | TURN relay (G30) |
| network blocks all UDP | ❌ nothing connects | TURN over TCP/443 (G30) |

## Reading guide

| Rule | Source |
|---|---|
| candidates, STUN, DHT, timers, failure message | ADR 0025 |
| one socket, split by magic | ADR 0025, ADR 0021 |
| what `wb join` accepts | ADR 0025, ADR 0016 |
| relay fallback | open (G30) |
