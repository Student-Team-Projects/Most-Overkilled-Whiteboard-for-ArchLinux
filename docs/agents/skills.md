# Agent skills

Which agent skills live in this repo, where they come from and how to update them.
When each one is used → `docs/agents/workflow.md`.

## Where

- Skills: `.agents/skills/<name>/SKILL.md`. Codex and opencode read this folder.
- Claude Code reads `.claude/skills/`, where each skill is a symlink into `.agents/skills/`.
- Upstream licences: `.agents/skills/licenses/`.

## Skills

| Skill | Source |
|---|---|
| `grilling`, `grill-with-docs`, `domain-modeling`, `to-spec`, `to-tickets`, `tdd`, `codebase-design` | mattpocock/skills |
| `rust-best-practices` (+ `references/`) | apollographql/skills |
| `review` | ours |

| Upstream | Commit | Licence |
|---|---|---|
| [mattpocock/skills](https://github.com/mattpocock/skills) | `4ae2317ddb22b046e26455e4f9c9b3aaaf62212d` (2026-10-31) | MIT → `mattpocock-skills.LICENSE` |
| [apollographql/skills](https://github.com/apollographql/skills) | `9dda1397730370aeb8df4ff8f2015c34c86b5003` (2026-10-30) | MIT → `apollographql-skills.LICENSE` |

Vendored skills are unmodified copies. Repo-specific rules go in `docs/agents/`, never into a vendored `SKILL.md`.

## How upstream names map here

| A skill says | Here it means |
|---|---|
| "run `/setup-matt-pocock-skills`" | already done: `docs/agents/issue-tracker.md` and `docs/agents/domain.md` |
| the `code-review` skill | our `review` skill |
| `to-spec`: publish the spec to the issue tracker | write `docs/features/<slug>/spec.md` with a Mermaid diagram (`docs/team.md`); its issue links to it |
| triage label `ready-for-agent` | no such label: Status `Todo` + type, area, milestone, Priority (`docs/board.md`) |

## Update

1. Clone the upstream repo and pick a new commit.
2. Replace each `.agents/skills/<name>/` folder with the upstream one; keep `review` and `licenses/`.
3. Change the commit in the table above; copy the licence again if it changed.
4. Read the upstream diff: a skill that now calls a new skill → vendor that one too, with its symlink.
