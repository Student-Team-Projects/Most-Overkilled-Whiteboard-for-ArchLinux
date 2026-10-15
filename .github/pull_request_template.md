<!-- Lines in these comments are instructions; they are hidden in the rendered PR. Rules: docs/team.md -->

Closes #
<!-- Every PR closes an issue (AGENTS.md → Backlog rules). A grill (G) issue closes only with an ADR or contract change. -->

## Summary

<!-- One or two sentences: what changes. Becomes the squash commit message with "Why".
     Summary and Why must make sense to a teammate who has not read the code: plain words, no internal
     names or jargon without a short explanation. Deep detail goes in the collapsed block below. -->

## Why

<!-- The reason, or a link to the spec / ADR / contract clause it implements. Same plain-words rule. -->

<details>
<summary>Details</summary>

<!-- Optional deep dive, collapsed by default: internals, edge cases, alternatives tried, numbers.
     Delete the whole block if there is nothing to add. -->

</details>

## How to review

<!-- Where to start reading, what is tricky, what is out of scope. Delete if the diff is obvious. -->

## Checklist

<!-- docs/team.md → Definition of done. Tick what applies; strike through (~~…~~) what does not, with a reason. -->
- [ ] Code is kosher (`docs/team.md` → Code standards)
- [ ] Tests for the logic; seam impls pass the shared seam suite
- [ ] A fast impl passes the oracle comparison and has a benchmark showing the win
- [ ] Docs touched in this PR: backlog updated, ADR if a decision was made
