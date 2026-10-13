# 0011 — Backlog lives in GitHub Issues + one Project board

Status: accepted · 2026-10-13

- **Context:** three people work in parallel; one markdown backlog means conflicts and no assignees.
  Deviates from the default "backlog in `docs/`" convention on purpose.
- **Decision:** every work item is a GitHub issue (labels `grill`/`task` + area, milestones M0–M5,
  native blocked-by), tracked on one Project board. ADRs, contracts, specs and the roadmap stay in
  the repo. Agents follow `AGENTS.md`; PRs carry `Closes #N` via the template — no CI check.
- **Why:** parallel work with owners and status in one place; the repo keeps what code review cites.
  Until the repo exists, `docs/backlog.md` is the staging list (T02 imports it, then deletes it).
