# Contract: `Camera`

Status: draft · World ↔ screen transform for pan and zoom. A concrete type (an affine matrix).
Format → `README.md` here.

## Operations

- `world_to_screen` — maps a world point to screen coordinates; does not report errors.
- `screen_to_world` — maps a screen point to world coordinates; does not report errors.
- `pan` — accepts a screen-space delta and moves the camera; does not report errors.
- `zoom_at` — accepts a screen anchor and zoom factor, keeping the world point under the anchor fixed; does not report errors.
- `visible_area` — accepts viewport size and returns its world-space rectangle; does not report errors.

## Guarantees

1. `screen_to_world(world_to_screen(p)) ≈ p` (within float tolerance) at any zoom in range.
2. `zoom_at` keeps the world point under `anchor` fixed.
3. Zoom is clamped to `[MIN_ZOOM, MAX_ZOOM]` = `[0.5, 2.0]` (ADR 0013).
4. `visible_area` contains every world point that maps inside the viewport.

## Not guaranteed

- Rotation (none) · smooth animation (UI concern).
- `WorldPos` is `i64`; screen math is `f32` relative to the camera centre, so precision does not drop far from the origin.

## Conformance tests (proptest)

`inverse_round_trip` (1) · `zoom_anchor_fixed` (2) · `zoom_clamped` (3) · `visible_area_covers_corners` (4)
