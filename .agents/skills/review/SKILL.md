---
name: review
description: Review a branch, PR or uncommitted diff in this repo before it reaches a human — edge cases and functionality first, then the code rules.
---

Humans judge readability. You hunt what breaks, then check the written rules.

## Steps

1. **Scope.** Read the diff: a branch → `git diff main...HEAD`; a PR → `gh pr diff <n>`; uncommitted → `git diff HEAD`
   plus every untracked file from `git status`. Then the issue it closes, and the contract in `docs/contracts/` for
   every seam it touches. Done when you can name each guarantee the diff must keep.
2. **Break it.** For every changed function, try the edges: empty and single-element input, maximum sizes and budgets,
   `i64`/`i32` overflow, duplicates and reordering, error paths, and each contract guarantee. Done when every guarantee
   has a test in the diff that would go red if it broke, or a finding saying which test is missing.
3. **Rules.** Check R1–R4 in `RULES.md` on every changed file. Done when every rule has been applied to every file.
4. **Report.** Findings ranked most severe first: `file:line` · what breaks or which rule · a one-line fix.
   Breakage outranks rules. Nothing found → say `clean`, and list the edges you tried.
