# Contract: `SpatialIndex`

Status: draft · "Which strokes are in this area?" — used by the eraser and the renderer.
Format → `README.md` here.

## Signature

```rust
pub trait SpatialIndex {
    fn insert(&mut self, id: StrokeId, bbox: Aabb);
    /// Returns whether `id` was present.
    fn remove(&mut self, id: StrokeId) -> bool;
    /// Appends every present id whose bbox intersects `area` (closed bounds) to `out`.
    fn query(&self, area: Aabb, out: &mut Vec<StrokeId>);
}
```

## Guarantees

1. `query` returns **exactly** the inserted-and-not-removed ids whose bbox intersects `area` — no
   misses, no extras, no duplicates.
2. Inserting an id already present replaces its bbox.
3. Works for any coordinates (infinite canvas): no fixed world bounds.

## Not guaranteed

- Order of results · exact geometry: it is bbox-only; the eraser does the precise hit test in `wb-core`.

## Impls

| Layer | Impl |
|---|---|
| 0 | `LinearIndex`: `Vec<(StrokeId, Aabb)>`, linear scan — **the oracle** |
| later | `QuadTree` (growing root for the infinite canvas) |

## Conformance tests

| Test | Checks |
|---|---|
| `matches_oracle` (proptest: random insert/remove/query vs `LinearIndex`) | 1, 2 |
| `far_coordinates` (±2⁶²) | 3 |
| bench: query time at 1k / 100k / 1M strokes | win vs oracle |
