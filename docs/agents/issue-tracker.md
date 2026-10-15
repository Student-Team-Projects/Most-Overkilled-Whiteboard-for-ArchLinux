# Issue tracker: GitHub Issues

Work items for this repo are GitHub issues on one Project board (ADR 0011). Use the `gh` CLI.
Repo: `Student-Team-Projects/Most-Overkilled-Whiteboard-for-ArchLinux` · Project: `6`, owner `Student-Team-Projects`.

## Conventions

| Thing | How |
|---|---|
| type | label `grill` or `task` (plus `bug`, `research` when needed) |
| area | label `core`, `codec`, `index`, `render`, `net`, `sim`, `app`, `infra`, `team` |
| milestone | GitHub milestone `M0`–`M5` (`docs/roadmap.md`) |
| blocked by | native issue dependency; also a `Blocked by: #N` line in the body |
| status | Project board column: Todo · Grilling · In progress · Review · Done |
| feature spec | `docs/features/<slug>/spec.md` in the repo, linked from the issue |

## When a skill says "publish to the issue tracker"

`gh issue create` with labels and milestone, then `gh project item-add 6 --owner Student-Team-Projects --url <issue-url>`.
Tickets from a spec are one issue each, linked from the spec — never a combined tickets file.

## When a skill says "fetch the relevant ticket"

`gh issue view <N> --comments`.

## Never

Create `docs/features/*/issues/` files or a markdown backlog — the board is the only list.
