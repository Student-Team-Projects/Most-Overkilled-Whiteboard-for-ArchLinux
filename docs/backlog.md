# Backlog — Most Overkilled Whiteboard

**Staging list** until the GitHub repo exists; then every row becomes an issue and this file is
deleted (T02). Milestones → `docs/roadmap.md` · decisions → `docs/adr/`.

Columns: **ID** · **What** · **Area** (label) · **After** (blocked by). Milestone = section.
A grill item is closed by an ADR or a contract change, never by a chat.

## Grill — product and UX (M0)

| ID | What | Area | After |
|---|---|---|---|
| G01 | Do peers see a stroke *while* it is being drawn (live ink), or only on pointer up? | app, net | — |
| G02 | Stroke width and eraser radius: world units (scale with zoom) or screen pixels? | render, core | — |
| G03 | What is 1 world unit; zoom range; does ±2³¹ suffice | core, render | — |
| G04 | Undo own stroke (= erase op): in or out of scope? | core | — |
| G05 | Session UX: CLI flags vs in-app session browser; one board per instance? | app | — |

## Grill — sync and protocol (M0–M1)

| ID | What | Area | After |
|---|---|---|---|
| G06 | Stroke bigger than one datagram: fragment, or cap points per stroke | net, codec | — |
| G07 | Save file: contents, version byte, restoring `seq` and Lamport on reopen | core, codec | — |
| G08 | Protocol version byte; malformed or foreign packets; mixed codec versions | net, codec | — |
| G09 | Version vector grows by one entry per run (random `PeerId`) — acceptable? | net | — |
| G10 | Anti-entropy pacing: how often, with whom, page size | net | G06 |

## Grill — network and distributed (M1–M4)

| ID | What | Area | After |
|---|---|---|---|
| G11 | Virtual network: broadcast/multicast? MTU? addressing? timeline — **ask the other team** | net, external | — |
| G12 | Discovery protocol on the virtual network | net | G11 |
| G13 | Gossip: fanout, push vs push-pull, what we measure | net | — |
| G14 | Localhost testing: port range, many instances on one machine | net, sim | — |

## Grill — performance (M3)

| ID | What | Area | After |
|---|---|---|---|
| G15 | Quadtree on an infinite canvas: growing root, strokes spanning many nodes | index | — |
| G16 | Chunk rendering: chunk size, detail per zoom level, cache memory budget | render | G03 |
| G17 | Existing stroke codec (not in this repo — ported in T23): fit with `i32` points, tolerance, licence | codec | — |
| G18 | Memory budget: what 1M strokes cost | core | G17 |

## Grill — sim, bench, runtime, team

| ID | What | Area | After |
|---|---|---|---|
| G19 | Simulator: in-process deterministic vs bots on many machines; workload + chaos config; metrics output | sim | G14 |
| G20 | Benchmark method: criterion, generated boards, how results are reported | sim | — |
| G21 | Net thread ↔ UI: channels, backpressure, repaint on remote op | app | — |
| G22 | Logging and observability: `tracing`, debug overlay | app | — |
| G23 | Who owns which feature first; review rotation | team | — |
| G24 | Agree the 9 contract drafts — split 3 / 3 / 3, each presents | team | — |

## Tasks — M0 Foundations

| ID | What | Area | After |
|---|---|---|---|
| T01 | GitHub org + repo; `main` protected: 1 review + CI required | infra | git access |
| T02 | GitHub Project + labels; import this file as issues; delete it | infra | T01 |
| T03 | Repo agent rules: `AGENTS.md` (+ `CLAUDE.md` import) — backlog lives in GitHub | infra | T02 |
| T04 | PR template: `Closes #`, contract clause touched, "reviewer read every line" | infra | T01 |
| T05 | Cargo workspace: 7 crates, workspace lints, pinned toolchain + egui/wgpu | infra | T01 |
| T06 | CI: fmt, clippy pedantic, test | infra | T05 |
| T07 | egui spike: each of us draws a line in a window (×3, throwaway) | app | — |
| T08 | Shared types in `wb-core` (`docs/contracts/README.md`) | core | G24, T05 |

## Tasks — M1 Walking skeleton (each needs its contract agreed, G24)

| ID | What | Area | After |
|---|---|---|---|
| T09 | `Board`: draw / erase / apply / merge / visible + proptest laws | core | T08 |
| T10 | `LinearIndex` + `SpatialIndex` conformance suite | index | T08 |
| T11 | `Camera` + proptests | render | T08 |
| T12 | `Raw` codec + `Message` enum + postcard framing with version byte | codec | T08, G08 |
| T13 | Window + `Naive` renderer + draw tool + colour picker | app, render | T09, T11 |
| T14 | Eraser tool: index query + precise hit test | app, core | T10, T13 |
| T15 | Pan / zoom input, Ctrl+scroll fallback | app | T11, T13 |
| T16 | `Udp` + `Sim` transports | net, sim | T08 |
| T17 | `Localhost` discovery + conformance suite | net | T16, G14 |
| T18 | `SendToAll` broadcast + conformance suite | net | T16 |
| T19 | Net thread driver + channels into the app | app, net | T17, T18, G21 |
| T20 | In-process sim: 3 peers converge, runs in CI | sim | T09, T16, T18 |
| T21 | CLI: port, board id, file | app | G05 |
| T22 | Demo: 2 laptops on the virtual network draw together | team | T19, G11 |

## Tasks — M3 (known early)

| ID | What | Area | After |
|---|---|---|---|
| T23 | Port the existing stroke compression code into `wb-codec` as the `Packed` codec | codec | G17, T12 |

M2–M5 are split into items when M1 lands — they depend on what grilling decides.

## Deliberately rejected or deferred

- Pixel eraser in v1 (ADR 0002) · TCP/QUIC (ADR 0003) · tokio (ADR 0004) · always-merge-on-open (ADR 0006).
- Tombstone garbage collection — not in v1.
- Security / auth, non-Linux platforms (README).
