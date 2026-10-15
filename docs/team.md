# Team guide

How the three of us work together: split, git flow, review, definition of done.
What we build → `docs/README.md` · how it is structured → `docs/architecture.md`.

## Team

Board: https://github.com/orgs/Student-Team-Projects/projects/6

| Person | GitHub | Leads | Reviews PRs of |
|---|---|---|---|
| Kacper | `poneciak57` | net (envelope, transports, STUN, rendezvous, manual join) | Filip |
| Mateusz | `hakiers` | core, pages model, save/load, PDF export, sim; quadtree after the MVP | Kacper |
| Filip | `fimpro` | GUI — window, tools, page view, start menu + connect modal | Mateusz |

Leads know their area best; features still go end-to-end. Assignments live on the board.

Board rules: `docs/board.md`.

## Split: feature teams

- Everyone takes features end-to-end (UI → core → net) from the milestone in `docs/roadmap.md`.
- Each feature gets a short spec in `docs/features/<slug>/spec.md` before code, with at least one
  Mermaid diagram (style and examples → `docs/design/`).
- **Contracts are shared property:** changing a seam trait or anything in `wb-core`'s public API
  needs agreement from all 3 and an ADR line. That is where feature teams collide — keep it explicit.

## Git flow

| Rule | Why |
|---|---|
| `main` is always green; short-lived branches `feat/…`, `fix/…`, `bench/…` | no long-lived divergence |
| PR ≤ ~1000 changed lines; bigger work is split | the size is a norm we hold each other to, not a tool check — the real rule is that the reviewer reads every line |
| 1 approving review from someone who did not write it; rotate reviewers | everyone understands every part |
| resolve a review thread once its finding is fixed; reply with the fixing commit | unresolved threads = open work |
| squash merge, message = what + why | readable history |

## Spikes

`spikes/<name>/` holds prep and learning code: its own Cargo project, **outside the workspace**, not
checked by CI, merged by PR. It uses stand-in types copied from the shared types in `docs/contracts/README.md` (T08 #32 moves
them into `wb-core`).

## CI

Every PR runs `ci` (`.github/workflows/ci.yml`: fmt, clippy, test; command list → `AGENTS.md` → Commands). After approval, the PR
goes through the GitHub merge queue, which re-runs `ci` on main + PR and merges only if green.
Benchmarks: method: G20 (#20), M4.

## Code standards

Code that meets all of these is **kosher** (`GLOSSARY.md`).

| Rule | Detail |
|---|---|
| understood, not just passing | every line is reviewed and understood by its stakeholder; no AI slop — if you can't explain it, it doesn't merge |
| lean | the smallest code that keeps the contract; no speculative abstraction |
| no panics in libs | no `unwrap`/`expect`/`panic!` outside tests and `wb-app` `main`; errors are `thiserror` enums |
| documented API | `#![deny(missing_docs)]` in library crates; seam methods cite their contract clause |
| docs without code | docs describe effects, guarantees and data in words; signatures live in code (rustdoc cites the contract clause); pseudocode only for algorithms and protocols |
| property tests | `proptest` for CRDT laws and oracle comparisons |
| pedantic lints | `clippy::pedantic` on; every `#[allow]` has a one-line reason |

## Definition of done

- Tests: unit tests for the logic; seam impls pass the shared seam suite.
- A fast impl also passes the oracle comparison and has a benchmark showing the win.
- Docs touched in the same PR: backlog updated, ADR if a decision was made.
- The reviewer can explain the change without the author in the room.

## Rhythm

- Weekly sync (30 min): what landed, what's blocked, next week's split. notes → `docs/journal.md`.
- Blocked > 1 day → say it in the chat, don't wait for the sync.
