# Agent workflow

How a feature goes from idea to merged code with agents, so code flows the same way for all 3.
Skills it uses → `docs/agents/skills.md` · code rules → `RULES.md` · human review, PRs → `docs/team.md`.

## The idea

A strong model and a human decide the *what*: decisions, spec, traits, contracts. A cheaper model writes
tests and code against them. The strong model checks the tests before any code exists, and reviews the
code before a human sees it. A trait with rustdoc and a conformance suite is the hand-off between the two.

## Stages

| # | Stage → artefact | Who · skill |
|---|---|---|
| 1 | Grill → answers in the G issue, ADR in `docs/adr/` | strong + human · `grill-with-docs` |
| 2 | Spec → `docs/features/<slug>/spec.md` | strong · `to-spec` |
| 3 | Trait + rustdoc + `docs/contracts/<seam>.md` | strong + all 3 humans · `codebase-design` |
| 4 | Tickets → GitHub issues | strong · `to-tickets` |
| 5 | Conformance tests, red → `tests/conformance/<seam>.rs` | cheaper · `tdd` |
| 6 | **Test gate** | strong · `review` |
| 7 | Implementation, green | cheaper · `tdd`, `rust-best-practices` |
| 8 | **Code review**, then a human | strong · `review`, `rust-best-practices` |

- Stage 3 changes a contract: all 3 humans + an ADR (`AGENTS.md` → Code rules); new constructors excepted (R4).
- A feature always gets stage 2 (`docs/team.md`). Work that is not a feature and changes no trait, contract or
  `wb-core` public API skips stages 2–4 and starts at 5 from its issue.
- Docs or tooling with no Rust code: only gate 8, without the `cargo` and Rust checks.
- Stage 5 departs from `tdd`'s one-test-at-a-time loop: the conformance suite is written whole, because it is
  the contract in code. Inside stage 7 the `tdd` loop applies as usual.

## Gates

Neither gate is optional. A PR that skipped one is not ready for a human.

| Gate | Passes when |
|---|---|
| test gate (6) | every guarantee in the contract and the trait's rustdoc (no seam: every acceptance criterion of the issue) has a test that fails on the **broken stub** below; tests use the names in `GLOSSARY.md` and follow R2/R3 |
| code review (8) | `review` reports `clean`, or each finding is fixed or answered; `rust-best-practices` finds nothing against `RULES.md`; `cargo` fmt, clippy and test pass (`AGENTS.md` → Commands) |

**Broken stub:** before an implementation exists, the suite runs against a stub impl in `tests/support/`
that breaks the guarantees on purpose. The gate's evidence is the test output: each guarantee has a test
that fails on the stub (not a compile error). The stub is deleted in stage 7.

## Rules for the cheaper model

- It does not edit traits, rustdoc on traits, contracts or conformance tests that passed the test gate.
  If one looks wrong, it stops and reports why — the strong model or a human decides.
- One ticket per session; the brief names the issue and, when there is one, the trait and its suite.
- Done = the suite is green, the three `cargo` checks pass and the definition of done in `docs/team.md`
  holds (a fast impl: oracle comparison + benchmark). It reports what it could not do.

## Model classes

The doc names classes, not models; the examples change with releases.

| Class | Used for | Examples (2026-11) |
|---|---|---|
| strong | stages 1–4, 6, 8 | Claude Opus · the top Codex model at high reasoning effort |
| cheaper | stages 5, 7 | Claude Sonnet or Haiku · Codex at low or medium effort |

Any tool works (Claude Code, Codex, opencode); they all read the same skills.
