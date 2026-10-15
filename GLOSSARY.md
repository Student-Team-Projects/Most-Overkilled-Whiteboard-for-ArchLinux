# Glossary

Shared words for code, specs and talks. Decisions behind them → `docs/adr/`.

| Term | Meaning |
|---|---|
| **Board** | one shared whiteboard, identified by `BoardId`; its state = strokes + tombstones |
| **Page** | an A4 rectangle in world units; MVP drawing happens only on pages (ADR 0019) |
| **STUN** | protocol to learn your public ip:port behind NAT; used with a rendezvous to connect peers |
| **Rendezvous** | the step where two peers exchange their public addresses before hole punching |
| **MVP** | See `docs/roadmap.md` (ADR 0019) |
| **Session** | a board currently live on the network (≥ 1 peer has it open) |
| **Peer** | one running app instance; random 64-bit `PeerId` per run |
| **Stroke** | one finished freehand line: encoded points + bbox + colour + width; immutable; a long line may be split into several strokes (ADR 0020) |
| **World unit** | 1/16 px at zoom 1.0; all coordinates are `i64` world units (ADR 0014) |
| **StrokeId** | the `OpId` `(PeerId, seq)` of the stroke's `Add` op — globally unique |
| **Lamport timestamp** | logical clock on each stroke: max seen + 1; with `PeerId` gives draw order |
| **Tombstone** | record that a stroke was erased; never removed in v1 |
| **Merge** | union of strokes and of tombstones; order and duplicates don't matter |
| **Fast-forward** | joining when local state ⊆ live state: just take the live state |
| **Diverged** | local state has things the live session lacks: user picks merge / copy / discard |
| **Anti-entropy** | periodic digest exchange between peers to fill in lost ops |
| **Gossip** | broadcast by forwarding to a few random peers instead of all |
| **Camera** | affine matrix world ↔ screen (pan + zoom) |
| **Chunk** | fixed-size world tile; unit of render caching |
| **Working set** | unpacked points of the strokes the screen needs; held by `StrokeCache` |
| **Seam** | a trait with a simple oracle impl and a later fast impl |
| **Contract** | the effect every impl of a seam must have; `docs/contracts/` |
| **Oracle** | the layer-0 impl that fast impls are tested against |
| **Kosher** | code meeting every rule in `docs/team.md` → Code standards |
