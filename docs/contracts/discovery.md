# Contract: `Discovery`

Status: draft · Who is out there and which board sessions exist (ADR 0006). Sans-IO.
Format → `README.md` here.

## Signature

```rust
pub trait Discovery {
    /// Periodic work: announce self, probe, expire silent peers.
    fn tick(&mut self, now: Time) -> Vec<Outgoing>;
    fn on_receive(&mut self, from: PeerAddr, msg: DiscoveryMsg, now: Time) -> Vec<Outgoing>;
    /// Peers currently believed alive, with the board each has open.
    fn peers(&self) -> &PeerSet;
    /// Board sessions seen on the network (for "scan for sessions").
    fn sessions(&self) -> Vec<SessionInfo>;
}
```

## Guarantees

1. Without loss, a reachable announcing peer appears in `peers()` within `T_FIND`.
2. A peer silent for `T_DEAD` is removed from `peers()`.
3. `peers()` never contains self.

## Not guaranteed

- That all peers have the same view at the same time · detection faster than the timeouts.

## Impls

| Layer | Impl |
|---|---|
| 0 | `Localhost`: probe a fixed port range on 127.0.0.1 every tick |
| later | virtual network discovery — depends on its capabilities (backlog P1) |

## Conformance tests

`finds_within_t_find` (1) · `expires_after_t_dead` (2) · `never_self` (3) — all in sim, N = 2…100.
