# Contract: `Transport`

Status: draft · Moves datagrams; nothing else (ADR 0003). Used only by drivers (net thread, sim),
never by protocol code. Format → `README.md` here.

## Signature

```rust
pub trait Transport {
    fn send(&mut self, to: PeerAddr, datagram: &[u8]) -> Result<(), TransportError>;
    /// Wait up to `timeout` for one datagram.
    fn recv(&mut self, timeout: Duration) -> Result<Option<(PeerAddr, Vec<u8>)>, TransportError>;
    fn max_datagram(&self) -> usize;
}
```

## Guarantees

1. A datagram arrives whole or not at all — never truncated or merged.
2. `send` of more than `max_datagram()` bytes is an `Err`, never a silent truncation.

## Not guaranteed

- Delivery, order, no duplicates — callers must survive loss, reordering and duplication.

## Impls

| Layer | Impl |
|---|---|
| 0 | `Udp` (std `UdpSocket`) · `Sim` (seeded loss, latency, duplication, partitions) |

## Conformance tests

`whole_or_nothing` (1) · `oversize_is_err` (2) · protocol suites run over `Sim` with chaos on.
