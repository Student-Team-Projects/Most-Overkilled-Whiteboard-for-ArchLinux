# Seam contracts

One file per seam: the effect every implementation must have. Written and agreed **before** any impl.
Seam list and layers → `docs/architecture.md` · decisions → `docs/adr/`.

## Rules

- The contract states the **weakest** guarantee all planned impls keep (layer 0 *and* the fast one).
  Code above the seam may rely only on what is written here.
- Every clause maps to a test in the seam's shared **conformance suite**; each impl runs it.
- Changing a contract = all 3 agree + ADR line (`docs/team.md`).

## Shared types (`wb-core`)

```rust
pub struct PeerId(pub u64);                         // random per run (ADR 0006)
pub struct BoardId(pub u128);
pub struct OpId { pub peer: PeerId, pub seq: u64 }  // ADR 0009
pub type StrokeId = OpId;                           // a stroke's id = its Add op's id
pub struct Lamport(pub u64);                        // ADR 0008
pub struct WorldPos { pub x: i64, pub y: i64 }      // 1 unit = 1/16 px at zoom 1; all math i64 (ADR 0014)
pub struct Aabb { pub min: WorldPos, pub max: WorldPos }
pub struct EncodedPoints(pub Vec<u8>);              // codec output: i64 origin + packed deltas; opaque to core
pub struct StrokePoints { pub origin: WorldPos, pub offsets: Vec<[i32; 2]> }  // decoded, on demand only
pub struct Stroke { pub id: StrokeId, pub lamport: Lamport, pub color: Rgba,
                    pub width: u16 /* world units, ADR 0013 */, pub bbox: Aabb, pub points: EncodedPoints }
pub enum Op { Add(Stroke), Erase { id: OpId, target: StrokeId } }  // grows by new variants (ADR 0015)
pub struct Time(pub u64);                           // ms; injected by the driver, never read from the OS
pub struct Outgoing { pub to: PeerAddr, pub msg: Message }  // sans-IO output
```

Coordinate rule: every coordinate computation is `i64`; `i32` exists only as decoded offsets.

## File shape

| Section | Contents |
|---|---|
| Signature | the Rust trait, doc comments only |
| Effect | what a call does, one line per method |
| Guarantees | what callers may rely on |
| Not guaranteed | what callers must not rely on (as important as guarantees) |
| Impls | layer 0 · later layers |
| Conformance tests | one line per test, mapped to a guarantee |

## Status

| Seam | File | Status |
|---|---|---|
| `Board` (type, not a trait) | `board.md` | draft |
| `StrokeCodec` | `stroke-codec.md` | draft |
| `SpatialIndex` | `spatial-index.md` | draft |
| `Camera` (type) | `camera.md` | draft |
| `StrokeRenderer` | `stroke-renderer.md` | draft |
| `StrokeCache` | `stroke-cache.md` | draft |
| `Transport` | `transport.md` | draft |
| `Discovery` | `discovery.md` | draft |
| `Broadcast` | `broadcast.md` | draft |
| `AntiEntropy` | `anti-entropy.md` | draft |

All drafts are reviewed and amended by the team in M0, then marked `agreed`.
