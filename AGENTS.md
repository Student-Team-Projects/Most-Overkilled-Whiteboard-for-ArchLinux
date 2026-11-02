# AGENTS.md — rules for every coding agent in this repo

Read by Claude Code (via `CLAUDE.md`), Codex and opencode. Team rules for humans → `docs/team.md`.

## Where things are

| Topic | Where |
|---|---|
| intent and scope | `docs/README.md` |
| milestones | `docs/roadmap.md` |
| crates, seams, layering rule | `docs/architecture.md` |
| diagrams: overview + complex topics | `docs/architecture.md`, `docs/design/` |
| what each seam must do | `docs/contracts/` |
| decisions | `docs/adr/` |
| journal | `docs/journal.md` |
| words | `GLOSSARY.md` |
| team workflow, definition of done | `docs/team.md` |
| code rules | `RULES.md` |
| **backlog** | **GitHub Issues + Project board** (ADR 0011) — see `docs/agents/issue-tracker.md` |

## Backlog rules (ADR 0011)

- Board rules (G/T ids, Status, Priority, templates): `docs/board.md`.
- Work items exist only as GitHub issues. Never write tasks or TODO lists into markdown files.
- Create, update and close items with `gh issue` / `gh project`; link blockers natively.
- Every PR body has `Closes #N` (in a PR stack, the others say `Part of #N` — `docs/team.md` → PR stacks). A grill issue closes only with an ADR or a contract change.
- Board: https://github.com/orgs/Student-Team-Projects/projects/6 · repo: `Student-Team-Projects/Most-Overkilled-Whiteboard-for-ArchLinux`.

## Code rules

- Follow `RULES.md` for all code changes.
- Changing a contract or `wb-core`'s public API needs all 3 humans + an ADR — stop and ask; additive constructors on `wb-core` types need no ADR (R4).
- For `spikes/` workspace boundaries, see `docs/team.md` → Spikes.

## PR reviews

- Automated reviewers (Codex) review every PR and re-review after each push; treat their findings like a human reviewer's.
- After fixing a finding: push, reply in the thread with the fixing commit, then resolve the thread. Leave unresolved only what is not fixed, with a one-line reason.
- At most 2 fix rounds per PR; anything found after that goes to the PR owner to decide (fix now, or a follow-up issue).

## Commands

- `cargo build --workspace`
- `cargo test --workspace`
- `cargo clippy --all-targets --workspace -- -D warnings`
- `cargo fmt --check`
- `cargo run -p wb-app`

## Agent skills

- Issue tracker: GitHub Issues. See `docs/agents/issue-tracker.md`.
- Domain docs: single-context (`GLOSSARY.md` + `docs/adr/`). See `docs/agents/domain.md`.
