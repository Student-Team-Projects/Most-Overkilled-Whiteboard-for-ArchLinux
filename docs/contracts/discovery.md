# Contract: `Discovery`

Status: agreed 2026-10-20 · Who is out there and which board sessions exist (ADR 0006). Sans-IO.
Format → `README.md` here.

## Operations

- `add_candidates` — accepts candidate addresses from a driver source (manual `ip:port`, DHT) and injected time; returns punch effects. Does not report errors.
- `tick` — accepts injected current time and returns outgoing announcements, probes, or expiry effects. Does not report errors.
- `on_receive` — accepts sender, discovery message, and injected time; returns outgoing replies. Does not report errors.
- `peers` — accepts no inputs and returns peers currently believed alive with their open board. Does not report errors.
- `sessions` — accepts no inputs and returns board sessions seen on the network. Does not report errors.

## Guarantees

1. Without loss, a reachable peer given as a candidate appears in `peers()` within `T_FIND`.
2. A peer silent for `T_DEAD` is removed from `peers()`.
3. `peers()` never contains self.

## Not guaranteed

- That all peers have the same view at the same time · detection faster than the timeouts.
- How long a source (the DHT) takes to produce candidates · in the MVP `sessions()` lists only joined or looked-up boards.

## Impls

| Layer | Impl |
|---|---|
| 0 | candidates from manual `ip:port` (+ `Localhost` for tests) |
| M1 | + STUN and punching; the DHT stays a driver-side candidate source (ADR 0025) |
| M4 | LAN / virtual network discovery |

## Conformance tests

`finds_within_t_find` (1) · `expires_after_t_dead` (2) · `never_self` (3) — all in sim, N = 2…100.
