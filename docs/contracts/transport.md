# Contract: `Transport`

Status: draft · Moves datagrams; nothing else (ADR 0003). Used only by drivers (net thread, sim),
never by protocol code. Format → `README.md` here.

## Operations

- `send` — accepts destination peer and datagram bytes; returns success or `TransportError`.
- `recv` — accepts a timeout; returns one peer and datagram, no datagram on timeout, or `TransportError`.
- `max_datagram` — accepts no inputs and returns the maximum supported datagram size; does not report errors.

## Guarantees

1. A datagram arrives whole or not at all — never truncated or merged.
2. `send` of more than `max_datagram()` bytes is an `Err`, never a silent truncation.
3. `max_datagram()` is 1200 bytes for both `Udp` and `Sim` until the virtual-network and STUN
   answers settle a larger value (G11/G29).

## Not guaranteed

- Delivery, order, no duplicates — callers must survive loss, reordering and duplication.

## Impls

| Layer | Impl |
|---|---|
| 0 | `Udp` (std `UdpSocket`) · `Sim` (seeded loss, latency, duplication, partitions) |

## Conformance tests

`whole_or_nothing` (1) · `oversize_is_err` (2) · `max_datagram_is_1200` on both `Udp` and `Sim` (3) · protocol suites run over `Sim` with chaos on.
