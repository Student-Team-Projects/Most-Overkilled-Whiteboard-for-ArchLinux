# Design: stroke lifecycle

One stroke from pointer to pixels on every peer, as a sequence diagram.
Overview → `docs/architecture.md` · convergence details → `sync-and-convergence.md` (this folder).

```mermaid
sequenceDiagram
    actor User
    participant App as wb-app tools
    participant Cam as Camera
    participant Codec as StrokeCodec
    participant Board as Board
    participant Index as SpatialIndex
    participant Net as net thread
    participant Peer as remote peer
    participant Cache as StrokeCache
    participant Rend as StrokeRenderer

    User->>App: pointer down / move, screen px
    App->>Cam: screen_to_world
    Cam-->>App: WorldPos, i64
    User->>App: pointer up
    App->>Codec: simplify + encode, once, author only
    Codec-->>App: one or more encoded pieces
    loop each piece
        App->>Board: draw(me, color, width, piece, bbox)
        Board-->>App: Op::Add with its own OpId + Lamport
        App->>Index: insert(id, bbox)
        App->>Net: op via channel
    end
    Net-)Peer: Broadcast, best-effort UDP
    Note over Net,Peer: lost? AntiEntropy heals it later
    Peer->>Peer: Board::apply(op), index insert, invalidate area
    App->>Rend: paint(frame)
    Rend->>Cache: get(id)
    Cache->>Codec: decode, only on first get
    Rend-->>User: frame
```

## Reading guide

| Step | Rule | Source |
|---|---|---|
| screen → world | all coordinates `i64`, 1 unit = 1/16 px | ADR 0014, `docs/contracts/camera.md` |
| simplify + encode | lossy, once, at the author; every peer stores identical bytes | ADR 0005, `docs/contracts/stroke-codec.md` |
| `Board::draw` | assigns `OpId` and Lamport; strokes stay encoded | ADR 0008, 0009, `docs/contracts/board.md` |
| Broadcast | best-effort; no delivery promise | ADR 0003, `docs/contracts/broadcast.md` |
| remote apply | idempotent, any order | ADR 0001, `docs/contracts/board.md` |
| `StrokeCache::get` | unpacked points only through the cache | ADR 0018, `docs/contracts/stroke-cache.md` |
| strokes appear on pen-up only | no live ink in v1 | ADR 0012 |
| oversized strokes split into independently ordered pieces | ADR 0020 | `docs/contracts/stroke-codec.md` |
