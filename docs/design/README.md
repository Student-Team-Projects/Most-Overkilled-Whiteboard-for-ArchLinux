# Design diagrams

Mermaid diagrams for complex topics; each file = 1–3 diagrams + a reading guide pointing at the ADRs.
Overview diagrams (crates, seams, runtime) → `docs/architecture.md` · decisions → `docs/adr/`.

| File | Shows |
|---|---|
| `stroke-lifecycle.md` | one stroke from pointer to every peer's screen |
| `sync-and-convergence.md` | board as a set, erase wins, anti-entropy healing loss |
| `join-flow.md` | join / reopen: fast-forward or the diverged dialog |
| `rendering.md` | M0 naive frame vs M3 chunk path |
| `devnet.md` | namespace dev network topology and lifecycle |
| `layering-and-testing.md` | when a fast impl may replace the oracle |

Rules: diagrams follow the docs, never lead them — an undecided part is drawn as "open (Gnn)".
Every feature spec in `docs/features/<slug>/spec.md` carries at least one diagram (`docs/team.md`).
