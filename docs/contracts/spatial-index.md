# Contract: `SpatialIndex`

Status: draft · "Which strokes are in this area?" — used by the eraser and the renderer.
Format → `README.md` here.

## Operations

- `insert` — accepts a stroke id and bounding box, inserting or replacing its bounds. Does not report errors.
- `remove` — accepts a stroke id and returns whether it was present. Does not report errors.
- `query` — accepts an area and appends every present id with an intersecting closed-bound box to the output. Does not report errors.

## Guarantees

1. `query` returns **exactly** the inserted-and-not-removed ids whose bbox intersects `area` — no
   misses, no extras, no duplicates.
2. Inserting an id already present replaces its bbox.
3. Works for any coordinates: no fixed world bounds, because world coordinates are `i64` (ADR 0014).

## Not guaranteed

- Order of results · exact geometry: it is bbox-only; the eraser does the precise hit test in `wb-core`.

## Impls

| Layer | Impl |
|---|---|
| 0 | `LinearIndex`: `Vec<(StrokeId, Aabb)>`, linear scan — **the oracle** |
| M4 | `QuadTree` (growing root for the infinite canvas) |

The unbounded-coordinate guarantee comes from `i64` world coordinates (ADR 0014); the quadtree
and growing root support the M4 infinite canvas.

## Conformance tests

| Test | Checks |
|---|---|
| `matches_oracle` (proptest: random insert/remove/query vs `LinearIndex`) | 1, 2 |
| `far_coordinates` (±2⁶²) | 3 |
| bench: query time at 1k / 100k / 1M strokes | win vs oracle |
