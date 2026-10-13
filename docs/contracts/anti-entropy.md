# Contract: `AntiEntropy`

Status: draft · The guarantee of convergence (ADR 0003): heals whatever `Broadcast` lost, and powers
late join and the join check (ADR 0006). Sans-IO. Format → `README.md` here.

## Signature

```rust
pub trait AntiEntropy {
    /// Periodic work: start a sync round with some peer.
    fn tick(&mut self, board: &Board, peers: &PeerSet, now: Time) -> Vec<Outgoing>;
    /// Handle a sync message; returns replies and the ops to apply locally.
    fn on_receive(&mut self, from: PeerAddr, msg: SyncMsg, board: &Board, now: Time) -> (Vec<Outgoing>, Vec<Op>);
    /// Compare local state with a remote summary (join flow).
    fn relation(&self, board: &Board, remote: &Summary) -> Relation; // Equal | Behind | Ahead | Diverged
}
```

## Guarantees

1. **Convergence:** if no new ops are made and every pair of peers is eventually connected
   (loss < 100 %, partitions heal), all boards become equal in finite time.
2. `relation` is correct: `Behind` iff local ⊂ remote, `Diverged` iff both have something the other lacks.
3. Every message fits in `Transport::max_datagram()` — large states are sent in pages.

## Not guaranteed

- Bandwidth or time bounds · progress during a partition (only after it heals).

## Impls

| Layer | Impl |
|---|---|
| 0 | `FullSet`: exchange full op-id lists, paged |
| later | `VersionVector`: per-author highest contiguous `seq` + gap lists (ADR 0009) |

## Conformance tests

| Test | Checks |
|---|---|
| `converges_under_chaos` (sim, seeded loss/latency/partition, N = 3…100) | 1 |
| `relation_matches_set_compare` (proptest vs brute-force set diff) | 2 |
| `pages_fit_datagram` on 100k-stroke boards | 3 |
| bench: bytes and rounds to converge | win vs `FullSet` |
