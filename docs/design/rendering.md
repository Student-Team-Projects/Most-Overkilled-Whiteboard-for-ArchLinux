# Design: rendering

How a frame is drawn: the M0 naive path (the oracle) and the M3 chunk path.
Overview → `docs/architecture.md` · contracts → `docs/contracts/stroke-renderer.md`, `docs/contracts/stroke-cache.md`.

## M0 — `Naive` renderer, every frame

```mermaid
flowchart LR
    cam["Camera::visible_area"] --> q["SpatialIndex::query → candidate ids"]
    q --> vis["keep visible strokes, in lamport, peer order"]
    vis --> get["StrokeCache::get → unpacked points"]
    get --> paint["egui painter draws each stroke via Camera transform"]
```

## M3 — chunk path

```mermaid
flowchart TD
    cam["Camera: visible area + margin"] --> res["resident chunks"]
    res --> setres["StrokeCache::set_resident → ChunkWorkingSet"]
    res --> hit{"tile cached and fresh?"}
    hit -- yes --> draw["draw tile texture"]
    hit -- no --> q["SpatialIndex::query chunk area"]
    q --> get["StrokeCache::get"]
    get --> tess["tessellate → render tile to wgpu texture"]
    tess --> store["store in tile cache"]
    store --> draw
    inv["Board change → invalidate area"] --> stale["mark tiles in area stale"]
    stale --> hit
```

## Reading guide

| Rule | Source |
|---|---|
| frame = visible strokes in viewport, `visible()` order | `docs/contracts/stroke-renderer.md` guarantee 1 |
| fast path matches `Naive` within pixel tolerance | `docs/contracts/stroke-renderer.md` guarantee 3 |
| points come only through `StrokeCache` | ADR 0018 |
| widths in world units, so tiles stay valid per zoom level | ADR 0013 |
| zoom range 0.5×–2× keeps the level pyramid small | ADR 0013 |
| tile size, levels, margin, byte budgets | open (G16) |
| egui + wgpu | ADR 0007, `docs/research/gui-stack.md` |
