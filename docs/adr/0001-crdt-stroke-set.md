# 0001 — Board state is a CRDT set of immutable strokes with tombstones

Status: accepted · 2026-10-13

- **Context:** no server; peers join late, lose packets, get partitioned; boards must still converge.
- **Decision:** a stroke is immutable once finished; `StrokeId = (PeerId, per-peer counter)`. Erase
  adds the id to a tombstone set. Board = (strokes, tombstones); merge = set union of both.
- **Why:** union is commutative, associative and idempotent — message order and duplicates do not
  matter, so gossip, late join, anti-entropy and file reopen are all the same operation: `merge`.
