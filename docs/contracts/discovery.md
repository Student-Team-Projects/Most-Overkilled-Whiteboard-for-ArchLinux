# Contract: `Discovery`

Status: accepted · 2026-11-16 · finds peers and board sessions through libp2p (ADR 0032). Sans-IO
protocol state; adapter lifecycle is surfaced through `Link`.

## Operations

- `tick` — accepts injected current time and returns protocol effects for rendezvous and expiry.
- `on_link_event` — accepts `PeerUp` or `PeerDown` and injected time; returns protocol effects.
- `peers` — returns peers currently believed alive and their open board.
- `sessions` — returns board sessions available for the current session code.

## Guarantees

1. `peers()` never contains self.
2. The adapter owns connection establishment and does not replay messages; lifecycle comes
   from `Link`, and `AntiEntropy` starts healing after `PeerUp`.
3. The Kademlia provider key is `SHA-256("wb-rendezvous-v1" || session_code)`. The code is never
   published or logged. DHT observers see the hash and peer addresses.

## Not guaranteed

- A synchronized peer view or a bound on discovery time.
- Session-code authorization: anyone with the code can join.
- Relay availability without the configured homelab node.

## Impls

| Layer | Impl |
|---|---|
| M1 | libp2p AutoNAT, Kademlia, Circuit Relay v2, DCUtR |
| M4 | LAN / virtual network discovery |

## Connection order

Direct dial; if AutoNAT reports private, reserve a relay; attempt DCUtR upgrade; stay on relay if
DCUtR fails. Configuration includes the homelab node and public Amino bootstrap nodes.
Without the homelab, Amino DHT and manual join remain available, but relay does not.
