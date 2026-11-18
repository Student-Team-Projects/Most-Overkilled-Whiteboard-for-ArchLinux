# Glossary

Shared words for code, specs and talks. Decisions behind them → `docs/adr/`.

| Term | Meaning |
|---|---|
| **Anti-entropy** | periodic exchange between peers to fill in missing operations |
| **AutoNAT** | libp2p protocol that estimates whether a peer is reachable from the public internet |
| **Board** | one shared whiteboard, identified by `BoardId`; its state = strokes + tombstones |
| **Bootstrap node** | a known libp2p peer that helps a node join the Kademlia network |
| **Camera** | affine matrix world ↔ screen (pan + zoom) |
| **Chunk** | fixed-size world tile; unit of render caching |
| **Contract** | the effect every impl of a seam must have; `docs/contracts/` |
| **DCUtR** | libp2p protocol that coordinates a direct connection attempt between peers already connected through a relay |
| **Diverged** | local state has things the live session lacks: user picks merge / copy / discard |
| **Fast-forward** | joining when local state ⊆ live state: union with the live state, no prompt |
| **Gossip** | broadcast by forwarding to a few random peers instead of all |
| **Kosher** | code meeting every rule in `RULES.md` |
| **Lamport timestamp** | logical clock on each stroke: max seen + 1; with `PeerId` gives draw order |
| **Link** | the seam for sending messages between peers and receiving peer lifecycle events (ADR 0032) |
| **Merge** | union of strokes and of tombstones; order and duplicates don't matter |
| **MVP** | See `docs/roadmap.md` (ADR 0019) |
| **Multiaddr** | a structured address that describes how to reach a network peer |
| **Oracle** | the layer-0 impl that fast impls are tested against |
| **Page** | an A4 rectangle in world units; MVP drawing happens only on pages (ADR 0019); pages are board state (ADR 0027) |
| **Peer** | one running app instance; first eight bytes of a hash of the ephemeral libp2p peer id per run (ADR 0032) |
| **PeerDown** | Link event that reports a peer is no longer considered connected |
| **PeerUp** | Link event that reports a peer is considered connected |
| **Printable area** | a page minus a 14 pt margin; the pen draws only inside it (ADR 0027) |
| **Relay** | a peer connection path forwarded by libp2p Circuit Relay v2 |
| **Rendezvous** | Kademlia provider lookup under `SHA-256("wb-rendezvous-v1" || session_code)` (ADR 0032) |
| **Seam** | a trait with a simple oracle impl and a later fast impl |
| **Session** | a board currently live on the network (≥ 1 peer has it open) |
| **Stamp** | a fresh `OpId` + Lamport that `Board::stamp` reserves for one local op; never reused |
| **Stroke** | one finished freehand line: encoded points + bbox + colour + width; immutable; a long line may be split into several strokes (ADR 0020) |
| **StrokeId** | the `OpId` `(PeerId, seq)` of the stroke's `Add` op — globally unique |
| **Tombstone** | record that a stroke was erased; never removed in v1 |
| **Working set** | unpacked points of the strokes the screen needs; held by `StrokeCache` |
| **World unit** | 1/16 px at zoom 1.0; all coordinates are `i64` world units (ADR 0014) |
