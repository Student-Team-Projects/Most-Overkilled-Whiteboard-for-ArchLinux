# Roadmap

Milestones until the end of semester and done-criteria. Status lives on the board. Per-feature specs →
`docs/features/<slug>/` · open items → https://github.com/orgs/Student-Team-Projects/projects/6.

Week 1 = 2026-10-13. Winter break (≈ 22.12–04.01) is buffer, not planned work.
MVP = M0–M1, functional first, layer 0 everywhere; M2 hardens and demos; M4 optimises.

| Milestone | Weeks | Done when |
|---|---|---|
| **M0 Local paged whiteboard** | 1–3 (→ 02.11) | CI green; contracts agreed; one window: draw in colours, whole-stroke eraser, pan/zoom over A4 pages, add page — all through layer 0; devnet |
| **M1 MVP: two peers over the internet** | 4–5 (→ 16.11) | ADR 0021 envelope + `Link` + `SendToAll`; libp2p connect and session-code rendezvous; manual LAN join; two peers on different networks draw together directly or via homelab relay; Noise encryption; devnet NAT scenario proves the path; start menu + connect modal; PDF export of non-empty pages |
| **M2 Bulletproof + stakeholder demo** | 8–10 (→ 21.12) | anti-entropy (`FullSet`); late join; save/load; join flow (ADR 0006); sim with loss/latency/partitions converges; bug bash; stakeholder demo of MVP before the break |
| **M4 Post-MVP: fast + distributed** | 11–12 (→ 11.01) | range hashes (G27); quadtree; chunk rendering; ChunkWorkingSet; LAN discovery; virtual network; gossip; 100-bot sim; benchmarks |
| **M5 Final demo** | 13–14 (→ 25.01) | demo script, report with numbers |

## Cut line

Cut M4 from the bottom up. M0–M2 (the MVP, hardened) is what we defend.
