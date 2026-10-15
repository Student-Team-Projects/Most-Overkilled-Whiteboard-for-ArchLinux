# Contract: `Broadcast`

Status: draft · How an op leaves this peer towards the others. Format → `README.md` here.

## Operations

- `broadcast` — accepts a locally authored operation and peer set; returns outgoing datagrams to spread it. Does not report errors.
- `on_receive` — accepts sender, received operation, and peer set; returns outgoing forwards, or none for an already seen operation. Does not report errors.

These operations are sans-IO and never touch a socket.

- `broadcast` — makes an attempt to get `op` to every peer in `peers`.
- `on_receive` — decides whether to forward; returns nothing for a message already seen.

## Guarantees

1. With no loss and a static `PeerSet`, every peer in the set receives `op` at least once.
2. An op (by `OpId`) is forwarded at most once per peer (no infinite loops), whatever the topology.
3. Never sends to self.

## Not guaranteed

- Delivery under loss or partitions — **anti-entropy** is what guarantees convergence, not this.
- Order between messages; exactly-once (duplicates are possible); latency bound.

## Impls

| Layer | Impl |
|---|---|
| 0 | `SendToAll`: one datagram per known peer, never forwards |
| later | `Gossip { fanout }`: send to k random peers, forward first-seen messages |

## Conformance tests

| Test | Checks |
|---|---|
| `reaches_all_without_loss` (sim, N = 2, 10, 100) | guarantee 1 |
| `terminates_on_any_topology` (random graphs, count sends) | guarantee 2 |
| `never_sends_to_self` | guarantee 3 |
