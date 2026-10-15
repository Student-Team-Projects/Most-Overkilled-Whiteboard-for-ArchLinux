# Most Overkilled Whiteboard

## Intent
<!-- In the user's own words. Do not polish. -->
- **Problem:** we want a P2P whiteboard — one GUI window, lightweight, aiming for Arch Linux. Very
  performant, lightweight and bulletproof.
- **Goal:** very simple spec — you can draw lines in different colors and delete them with an eraser
  on an infinite canvas. Squeezed to maximum: quadtrees, chunk rendering, matrix for movement on the
  board, distributed algorithm for connecting and discovery, gossip protocol for broadcast.
- **Constraints:**
  - Rust. Team of 3.
  - Another team builds the Arch Linux virtual network — we assume all devices are connected and we
    can send to them like in a local network. We do not handle connecting, just P2P whiteboard.
  - Layered plan: first everything as simple as possible but with good engineering practices, so the
    structure is final. Later reimplement components to be more performant, each testable in
    isolation (e.g. quadtree starts as a vector with linear search).
  - Deadline: end of semester (~3 months, ~mid-January 2027).
- **Out of scope:** security / auth (trust everyone on the virtual network) · non-Linux platforms.
  Shapes, text, images — "kind of in scope, nice as an extension, but we aim for drawing".
- **Done means (final demo):** all of —
  - 3+ peers draw live, boards converge.
  - Survives chaos + benchmarks: "simulation of the states so we simulate chaos on multiple machines
    simulating lets say 100 users drawing some lines deleting, causing a lot of events, and we test
    and benchmark".
  - Huge board stays smooth — on prepared generated examples.
  - Save to disk: "we want to be able to save it to the disk".
- **Rejected / deferred:** pixel eraser (ADR 0002) · TCP/QUIC (0003) · tokio (0004) ·
  always-merge-on-open (0006) · tombstone GC in v1.
- **Open questions:** what happens when you reopen a board from file — "not yet decided".

## Where things are

| Looking for | Where |
|---|---|
| milestones, status | `docs/roadmap.md` |
| crates, seams, layering rule | `docs/architecture.md` |
| diagrams of complex topics | `docs/design/` |
| how the team works | `docs/team.md` |
| what is left, what needs deciding | GitHub board: https://github.com/users/poneciak57/projects/3 (ADR 0011) |
| research notes | `docs/research/` |
| decisions | `docs/adr/` |
| feature specs and tickets | `docs/features/` |
| domain vocabulary | `GLOSSARY.md` |
