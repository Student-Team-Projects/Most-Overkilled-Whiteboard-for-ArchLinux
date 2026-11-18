# Contract: `Link`

Status: accepted · 2026-11-16 · carries protocol messages between peers (ADR 0032). Used by
protocol drivers and `Sim`; protocol logic never touches the libp2p swarm.

## Operations

- `send` — accepts destination `PeerId` and one message of at most 1200 bytes; reports local
  acceptance or a local error, not remote delivery.
- Events — `Received(peer, message)`, `PeerUp(peer)`, and `PeerDown(peer)` report a complete
  incoming message and the adapter's current peer lifecycle view. The driver feeds these events
  to the sans-IO core.

## Guarantees

1. A message accepted by `send` is delivered as one complete message or not delivered; it is never
   merged with another message or silently truncated.
2. The maximum message is 1200 bytes. The length prefix on the libp2p stream is outside this cap;
   the ADR 0021 envelope is the capped unit.
3. Messages use a u16 big-endian length prefix. A decoded length above 1200 resets the stream;
   framing cannot resynchronize. A malformed envelope within the limit is dropped and counted by
   the T26 reject counters while the stream stays open.
4. The adapter reconnects with backoff of about 1 s to 30 s and forgets a peer after about 2 min
   without contact. It emits lifecycle events and does not replay messages.

## Not guaranteed

- Delivery, ordering, or duplicate suppression. Protocols tolerate loss, reordering, and repeats.
- A bound on peer detection or reconnection time.

## Impls

| Layer | Impl |
|---|---|
| M1 | libp2p adapter · `Sim` with seeded loss, duplication, reordering, partitions, and lifecycle events |

## Conformance tests

| Guarantee | Coverage |
|---|---|
| 1 | Sim conformance: `Received` complete messages under seeded loss, duplication, and reordering |
| 2 | Devnet: an envelope up to 1200 bytes is delivered; an over-limit frame resets its stream |
| 3 | Devnet: malformed envelope is counted and dropped; stream remains up |
| 4 | Sim conformance: up/down events under partitions. Devnet: reconnect backoff, ~2 min expiry, and no replay after reconnect |
| 5 | Devnet integration (ADR 0017): direct, DCUtR, and relay paths |

## Thread boundary

The GUI and net thread use bounded `tokio::sync::mpsc` channels in both directions. The UI never
blocks: it uses `try_send` (drop and log when full) and `try_recv` each frame. Net events request
an egui repaint. The net thread owns the Tokio runtime and swarm (ADR 0032).
