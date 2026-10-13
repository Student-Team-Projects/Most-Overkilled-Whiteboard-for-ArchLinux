# 0006 — Joining a board: fast-forward, or choose when diverged

Status: accepted · 2026-10-13

- **Context:** a peer may hold local state for a board (from a file or offline work) that differs
  from the live session.
- **Decision:** peers can scan the network for live board sessions. On join, compare local state L
  with live state R:
  - L ⊆ R → **fast-forward**: just take R, no prompt.
  - L has strokes/tombstones R lacks → **diverged**: the user chooses *merge* (union, ADR 0001),
    *open as copy* (new board_id) or *discard local* (take R).
- **Why:** git-like and very simple; union is always safe, the prompt only guards against unwanted
  visual overlap. Each session gets a random 64-bit PeerId so reopened files never reuse stroke ids.
