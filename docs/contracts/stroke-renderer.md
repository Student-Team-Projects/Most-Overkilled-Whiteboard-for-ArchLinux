# Contract: `StrokeRenderer`

Status: draft · Puts the visible part of the board on screen. Format → `README.md` here.

## Signature

```rust
pub trait StrokeRenderer {
    /// The board changed inside `area` (add or erase); cached output there is stale.
    fn invalidate(&mut self, area: Aabb);
    /// Draw the visible strokes for this frame.
    fn paint(&mut self, painter: &egui::Painter, camera: &Camera, board: &Board, index: &dyn SpatialIndex);
}
```

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
