# Board rules

How we use the GitHub board: https://github.com/orgs/Student-Team-Projects/projects/6 (ADR 0011).
Mirrored into the project README. Team workflow → `docs/team.md` · agent rules → `AGENTS.md`.

## Item types

| Prefix | Label | Is | Closes with |
|---|---|---|---|
| **G**nn | `grill` | a design decision to make | a merged ADR or contract change, linked in the closing comment |
| **T**nn | `task` | work with a concrete deliverable | a PR with `Closes #N` |

Numbers are never reused. Next free: see the highest G/T on the board. Title = `Gnn short question`.
Bodies follow the issue templates (grill: Question · Why · Options · Recommendation · Deliverable ·
Done when · Context · Blocked by; task: Goal · Deliverable · Acceptance · Context · Blocked by).

## Fields

| Field | Values | Meaning |
|---|---|---|
| Status | Todo | not started |
| | Grilling | decision being discussed (grill items only) |
| | In progress | someone is on it — has an assignee |
| | Review | PR open; reviewer reads every line |
| | Done | closed |
| Priority | P0 | blocks others now — this week |
| | P1 | needed for the current milestone |
| | P2 | later milestone — do not start yet |
| Milestone | M0–M5 | `docs/roadmap.md` |
| Labels | type + area | area: `core` `codec` `index` `render` `net` `sim` `app` `infra` `team` `external` |

## Rules

1. Every issue has type label, area label, milestone, Status and Priority.
2. Don't start a blocked item (GitHub "Blocked by"). Unblocked P0 first, then P1.
3. In progress needs an assignee. At most 2 items In progress per person.
4. Move your own cards. Status changes are not announced in chat — the board is the truth.
5. New work → new issue from a template, never a TODO in markdown or code.
6. Grilling outcome = ADR / contract PR; then close the G item with the link.

## Views

| View | Shows |
|---|---|
| Board | everything by Status |
| Current milestone | board filtered to the active milestone |
| Grill | open decisions |
| My work | your assigned items |
| All (table) | every item with all fields |
| Roadmap | items over milestone dates |
