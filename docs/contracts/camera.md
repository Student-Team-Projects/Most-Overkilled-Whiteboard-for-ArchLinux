# Contract: `Camera`

Status: draft · World ↔ screen transform for pan and zoom. A concrete type (an affine matrix).
Format → `README.md` here.

## Signature

```rust
impl Camera {
    pub fn world_to_screen(&self, p: WorldPos) -> ScreenPos;
    pub fn screen_to_world(&self, p: ScreenPos) -> WorldPos;
    pub fn pan(&mut self, delta: ScreenVec);
    /// Zoom by `factor` keeping the world point under `anchor` fixed on screen.
    pub fn zoom_at(&mut self, anchor: ScreenPos, factor: f32);
    /// World rectangle visible in a viewport of `size`.
    pub fn visible_area(&self, size: ScreenVec) -> Aabb;
}
```

## Guarantees

1. `screen_to_world(world_to_screen(p)) ≈ p` (within float tolerance) at any zoom in range.
2. `zoom_at` keeps the world point under `anchor` fixed.
3. Zoom is clamped to `[MIN_ZOOM, MAX_ZOOM]`.
4. `visible_area` contains every world point that maps inside the viewport.

## Not guaranteed

- Rotation (none) · smooth animation (UI concern).

## Conformance tests (proptest)

`inverse_round_trip` (1) · `zoom_anchor_fixed` (2) · `zoom_clamped` (3) · `visible_area_covers_corners` (4)
