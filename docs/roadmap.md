# Roadmap

Milestones until the end of semester, done-criteria and status. Per-feature specs →
`docs/features/<slug>/` · open items → https://github.com/users/poneciak57/projects/3.

Week 1 = 2026-10-13. Winter break (≈ 22.12–04.01) is buffer, not planned work.

| Milestone | Weeks | Done when | Status |
|---|---|---|---|
| **M0 Foundations** | 1–2 (→ 26.10) | repo + workspace + CI green; seam traits in `wb-core` agreed by all 3; glossary; everyone has drawn a line in an egui spike | next |
| **M1 Walking skeleton** | 3–5 (→ 16.11) | one window: draw, colours, whole-stroke eraser, pan/zoom (camera matrix); 2 laptops on the virtual network draw together (simple `Discovery`, UDP, send-to-all); 3-peer in-process sim test passes; `devnet.sh` runs several peers on one laptop (ADR 0017) | — |
| **M2 Bulletproof** | 6–7 (→ 30.11) | anti-entropy; late join; save/load; join flow (fast-forward / diverged, ADR 0006); sim with loss + latency + partitions proves convergence on seeded random runs; headless bot peers | — |
| **M3 Fast** | 8–10 (→ 21.12) | quadtree; chunk rendering; stroke codec; generated huge boards (100k+ strokes) pan/zoom smoothly; benchmarks: layer 0 vs fast for each seam | — |
| **M4 Distributed** | 11–12 (→ ~11.01) | scan network for board sessions; discovery; gossip broadcast; ~100 bot peers on several machines under chaos converge; metrics (convergence time, bytes sent) | — |
| **M5 Demo** | 13–14 (→ ~25.01) | hardening, demo script, report with benchmark numbers | — |

## Cut line

If time runs out, cut from the bottom of M4 upward (gossip before discovery). M0–M2 + the quadtree
and chunk rendering from M3 are the minimum we defend.
