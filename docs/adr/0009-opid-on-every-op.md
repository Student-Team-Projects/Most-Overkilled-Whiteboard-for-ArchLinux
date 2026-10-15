# 0009 — Every op (add and erase) has an OpId = (PeerId, seq)

Status: accepted · 2026-10-13

- **Context:** gossip must dedup erases too; anti-entropy digests grow with the number of ops.
- **Decision:** each author numbers all its ops 0, 1, 2…; a stroke's id is its `Add` op's id.
- **Why:** one dedup key for everything, and each author's ops form a log, so anti-entropy can later
  summarise state as a version vector (highest contiguous `seq` per author) instead of id lists;
  id pages remain the fallback when vectors differ or do not fit.
