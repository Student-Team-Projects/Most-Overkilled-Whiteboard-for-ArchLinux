# Roadmap

Milestones until the end of semester, done-criteria and status. Per-feature specs →
`docs/features/<slug>/` · open items → https://github.com/orgs/Student-Team-Projects/projects/6.

Week 1 = 2026-10-13. Winter break (≈ 22.12–04.01) is buffer, not planned work.

| Milestone | Weeks | Done when | Status |
|---|---|---|---|
| **M0 Local whiteboard + test env** | 1–3 (→ 02.11) | workspace + CI green; contracts agreed by all 3; **one local window**: draw in colours, whole-stroke eraser, pan/zoom — all through the seams (`Board`, `LinearIndex`, `Camera`, `Raw` codec, `StrokeCache`, `Naive` renderer); **test env**: `devnet.sh` (ADR 0017), conformance-suite pattern, CI | next |
| **M1 Networked skeleton** | 4–5 (→ 16.11) | transports (`Udp`, `Sim`), message envelope, `Localhost` discovery, `SendToAll`, net thread, CLI (ADR 0016); several peers draw together in the devnet; 3-peer in-process sim passes in CI; 2 laptops on the virtual network if it is ready | — |
| **M2 Bulletproof** | 6–7 (→ 30.11) | anti-entropy; late join; save/load; join flow (fast-forward / diverged, ADR 0006); sim with loss + latency + partitions proves convergence on seeded random runs; headless bot peers | — |
| **M3 Fast** | 8–10 (→ 21.12) | quadtree; chunk rendering; stroke codec; generated huge boards (100k+ strokes) pan/zoom smoothly; benchmarks: layer 0 vs fast for each seam | — |
| **M4 Distributed** | 11–12 (→ ~11.01) | scan network for board sessions; discovery; gossip broadcast; ~100 bot peers on several machines under chaos converge; metrics (convergence time, bytes sent) | — |
| **M5 Demo** | 13–14 (→ ~25.01) | hardening, demo script, report with benchmark numbers | — |

## Cut line

If time runs out, cut from the bottom of M4 upward (gossip before discovery). M0–M2 + the quadtree
and chunk rendering from M3 are the minimum we defend.
