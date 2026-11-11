# Contract: `Transport`

Status: agreed 2026-10-20 · Moves datagrams; nothing else (ADR 0003). Used only by drivers (net thread, sim),
never by protocol code. Format → `README.md` here.

## Operations

- `send` — accepts a destination address (`SocketAddr`) and datagram bytes; returns success or `TransportError`.
- `recv` — accepts a timeout; returns one sender address and datagram, no datagram on timeout, or `TransportError`.
- `max_datagram` — accepts no inputs and returns the maximum supported datagram size; does not report errors.

## Guarantees

1. A datagram arrives whole or not at all — never truncated or merged.
2. `send` of more than `max_datagram()` bytes is an `Err`, never a silent truncation.
3. `max_datagram()` is 1200 bytes for both `Udp` and `Sim`, one constant (ADR 0025), until the
   virtual-network answer settles a larger value (G11).

Addresses, not `PeerId`s: a UDP datagram does not say which peer sent it. The driver maps
`PeerId` ↔ address from what discovery learned.

`Sim` is one in-memory network (`SimNet`) whose clock the sim driver moves; time never passes
inside `recv`, so `Sim::recv` returns only datagrams that already arrived and never waits. Chaos is
drawn from a seed: loss and duplication chances, latency plus uniform jitter (which reorders), and
a partition that loses every datagram sent across it until healed.

## Not guaranteed

- Delivery, order, no duplicates — callers must survive loss, reordering and duplication.

## Impls

| Layer | Impl |
|---|---|
| 0 | `Udp` (std `UdpSocket`) · `Sim` (seeded loss, latency, jitter, duplication, partitions) |

## Conformance tests

`whole_or_nothing` (1) · `oversize_is_err` (2) · `max_datagram_is_1200` on both `Udp` and `Sim` (3) · protocol suites run over `Sim` with chaos on.
