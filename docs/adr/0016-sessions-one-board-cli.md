# 0016 — One board per instance; named boards; CLI to start

Status: accepted · 2026-10-14 · closes G05

- **Decision:**
  - One board per app instance; another board = another instance.
  - A board has a random `BoardId` and a human name; `list` shows name · short id · peer count;
    `join` matches a name or an id prefix.
  - CLI: `wb new <name>` · `wb join <name|id>` · `wb open <file>` · `wb list`. Ctrl+S saves.
  - The "diverged" choice (ADR 0006) is an in-app dialog. An in-app session browser is optional (M4).
- **Why:** smallest UX that covers create / join / reopen; the dialog works without a terminal.
