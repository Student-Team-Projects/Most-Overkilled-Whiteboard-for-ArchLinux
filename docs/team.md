# Team guide

How the three of us work together: split, git flow, review, definition of done.
What we build → `docs/README.md` · how it is structured → `docs/architecture.md`.

## Team

Mateusz · Filip · poneciak57. Board: https://github.com/users/poneciak57/projects/3

Board rules: `docs/board.md`.

## Split: feature teams

- Everyone takes features end-to-end (UI → core → net) from the milestone in `docs/roadmap.md`.
- Each feature gets a short spec in `docs/features/<slug>/spec.md` before code.
- **Contracts are shared property:** changing a seam trait or anything in `wb-core`'s public API
  needs agreement from all 3 and an ADR line. That is where feature teams collide — keep it explicit.

## Git flow

| Rule | Why |
|---|---|
| `main` is always green; short-lived branches `feat/…`, `fix/…`, `bench/…` | no long-lived divergence |
| PR ≤ ~1000 changed lines; bigger work is split | the size is a norm we hold each other to, not a tool check — the real rule is that the reviewer reads every line |
| 1 approving review from someone who did not write it; rotate reviewers | everyone understands every part |
| squash merge, message = what + why | readable history |

## CI (every PR)

`cargo fmt --check` · `cargo clippy --all-targets -- -D warnings -W clippy::pedantic` ·
`cargo test --workspace` (incl. proptest) ·
seeded sim convergence test. Benchmarks run manually, results committed to `docs/benchmarks/`.

## Code standards

| Rule | Detail |
|---|---|
| understood, not just passing | every line is reviewed and understood by its stakeholder; no AI slop — if you can't explain it, it doesn't merge |
| lean | the smallest code that keeps the contract; no speculative abstraction |
| no panics in libs | no `unwrap`/`expect`/`panic!` outside tests and `wb-app` `main`; errors are `thiserror` enums |
| documented API | `#![deny(missing_docs)]` in library crates; seam methods cite their contract clause |
| property tests | `proptest` for CRDT laws and oracle comparisons |
| pedantic lints | `clippy::pedantic` on; every `#[allow]` has a one-line reason |

## Definition of done

- Tests: unit tests for the logic; seam impls pass the shared seam suite.
- A fast impl also passes the oracle comparison and has a benchmark showing the win.
- Docs touched in the same PR: backlog updated, ADR if a decision was made.
- The reviewer can explain the change without the author in the room.

## Rhythm

- Weekly sync (30 min): what landed, what's blocked, next week's split. Notes → `docs/journal.md`.
- Blocked > 1 day → say it in the chat, don't wait for the sync.
