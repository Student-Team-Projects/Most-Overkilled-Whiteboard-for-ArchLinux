# Contract: `StrokeRenderer`

Status: draft · Puts the visible part of the board on screen. Format → `README.md` here.

## Operations

- `invalidate` — accepts an area changed by an add or erase and marks cached output there stale. Does not report errors.
- `paint` — accepts the painter, camera, board, spatial index, and stroke cache, then draws the visible strokes for the frame; points are obtained only through the cache (ADR 0018). Does not report errors.

## Guarantees

1. The frame shows exactly the strokes of `board.visible()` that intersect the viewport, in
   `visible()` order, transformed by `camera`.
2. After `invalidate(area)`, the next `paint` reflects every change inside `area`.
3. Output matches the naive renderer within a small pixel tolerance (anti-aliasing may differ).

## Not guaranteed

- A frame-time bound (that is the benchmark, not the contract) · identical pixels across GPUs.

## Impls

| Layer | Impl |
|---|---|
| 0 | `Naive`: egui painter, every visible stroke, every frame — **the oracle** |
| later | `ChunkMesh`: tessellated meshes cached per chunk · then `ChunkTexture`: chunks rendered to wgpu textures per zoom level |

## Conformance tests

| Test | Checks |
|---|---|
| snapshot vs `Naive` on generated boards (`egui_kittest`, unverified fit) | 1, 3 |
| `invalidate_then_paint` shows add and erase | 2 |
| bench: frame time pan/zoom on 10k / 100k strokes | win vs oracle |
