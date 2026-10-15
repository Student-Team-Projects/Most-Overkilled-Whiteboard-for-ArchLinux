# Design: sync and convergence

Why every peer ends with the same picture: the `Board` set, erase-wins, and anti-entropy healing loss.
Overview → `docs/architecture.md` · contracts → `docs/contracts/board.md`, `docs/contracts/anti-entropy.md`.

## The board is a set — merge is union

```mermaid
flowchart LR
    A["Replica A: strokes a1, b1 · tombstones none"] --> M["merge = union of strokes, union of tombstones"]
    B["Replica B: strokes b1, c1 · tombstones b1"] --> M
    M --> R["strokes a1, b1, c1 · tombstones b1"]
    R --> V["visible = strokes minus tombstones, sorted by lamport, peer → a1, c1"]
```

## Erase wins, whatever the arrival order

```mermaid
sequenceDiagram
    participant A as peer A
    participant B as peer B
    participant C as peer C
    A-)B: Add b1
    B-)C: Erase b1
    Note over C: Erase arrives first: tombstone kept
    A-)C: Add b1, late
    Note over C: b1 stays invisible, same as on A and B
```

## Broadcast may lose; anti-entropy heals

```mermaid
sequenceDiagram
    participant A as peer A
    participant B as peer B
    participant C as peer C
    A-)C: Broadcast op o1
    A-xB: Broadcast op o1, lost
    Note over B: B is missing o1
    B->>A: AntiEntropy round: comparison summary
    A->>B: comparison identifies missing o1
    A->>B: send o1 after comparison (ordinary sync)
    Note over A,C: all equal once no new ops and partitions heal
```

## Reading guide

| Rule | Source |
|---|---|
| state = strokes + tombstones; merge = union | ADR 0001, `docs/contracts/board.md` guarantee 1 |
| erase wins | ADR 0001, `docs/contracts/board.md` guarantee 3 |
| draw order `(lamport, peer)` | ADR 0008, `docs/contracts/board.md` guarantee 4 |
| every op has an `OpId` (dedup key) | ADR 0009 |
| broadcast is best-effort | `docs/contracts/broadcast.md` |
| convergence comes from anti-entropy | ADR 0003, `docs/contracts/anti-entropy.md` guarantee 1 |
| round pacing, peer choice, page size | ADR 0023 |
| version-vector growth per run | ADR 0022 |
