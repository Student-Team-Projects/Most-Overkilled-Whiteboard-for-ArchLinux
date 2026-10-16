# Contract: `Discovery`

Status: draft · Who is out there and which board sessions exist (ADR 0006). Sans-IO.
Format → `README.md` here.

## Operations

- `tick` — accepts injected current time and returns outgoing announcements, probes, or expiry effects. Does not report errors.
- `on_receive` — accepts sender, discovery message, and injected time; returns outgoing replies. Does not report errors.
- `peers` — accepts no inputs and returns peers currently believed alive with their open board. Does not report errors.
- `sessions` — accepts no inputs and returns board sessions seen on the network. Does not report errors.

## Guarantees

1. Without loss, a reachable announcing peer appears in `peers()` within `T_FIND`.
2. A peer silent for `T_DEAD` is removed from `peers()`.
3. `peers()` never contains self.

## Not guaranteed

- That all peers have the same view at the same time · detection faster than the timeouts.
- DHT impl: no bound on lookup time, so guarantee 1 holds only for `Manual`; `sessions()` lists only the looked-up board.

## Impls

| Layer | Impl |
|---|---|
| 0 | Manual `ip:port`, given at construction (+ `Localhost` for tests) |
| M1 | STUN + Mainline DHT rendezvous (ADR 0025) |
| M4 | LAN / virtual network discovery |

## Conformance tests

`finds_within_t_find` (1) · `expires_after_t_dead` (2) · `never_self` (3) — all in sim, N = 2…100.
