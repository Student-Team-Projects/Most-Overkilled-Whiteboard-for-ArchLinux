# AGENTS.md — rules for every coding agent in this repo

Read by Claude Code (via `CLAUDE.md`), Codex and opencode. Team rules for humans → `docs/team.md`.

## Where things are

| Topic | Where |
|---|---|
| intent and scope | `docs/README.md` |
| milestones | `docs/roadmap.md` |
| crates, seams, layering rule | `docs/architecture.md` |
| what each seam must do | `docs/contracts/` |
| decisions | `docs/adr/` |
| words | `GLOSSARY.md` |
| team workflow, code standards, definition of done | `docs/team.md` |
| **backlog** | **GitHub Issues + Project board** (ADR 0011) — see `docs/agents/issue-tracker.md` |

## Backlog rules (ADR 0011)

- Work items exist only as GitHub issues. Never write tasks or TODO lists into markdown files.
- Create, update and close items with `gh issue` / `gh project`; link blockers natively.
- Every PR body has `Closes #N`. A grill issue closes only with an ADR or a contract change.
- Until the repo exists, `docs/backlog.md` is the staging list.

## Code rules

- Code above a seam relies only on what its contract in `docs/contracts/` states.
- Changing a contract or `wb-core`'s public API needs all 3 humans + an ADR — stop and ask.
- Lean, minimal diffs: every line must be explainable by the human who owns the PR.
  No speculative abstraction, no drive-by refactors.
- Full standards (no panics in libs, pedantic clippy, docs, proptest) → `docs/team.md`.

## Agent skills

- Issue tracker: GitHub Issues. See `docs/agents/issue-tracker.md`.
- Domain docs: single-context (`GLOSSARY.md` + `docs/adr/`). See `docs/agents/domain.md`.
