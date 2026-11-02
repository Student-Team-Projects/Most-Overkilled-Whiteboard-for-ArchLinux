# Seam contracts

One file per seam: the effect every implementation must have. Written and agreed **before** any impl.
Seam list and layers → `docs/architecture.md` · decisions → `docs/adr/`.
Write each contract in prose; the trait in code is the signature, documented per `RULES.md` R1.

## Rules

- The contract states the **weakest** guarantee all planned impls keep (layer 0 *and* the fast one).
  Code above the seam may rely only on what is written here.
- Every clause maps to a test in the seam's shared **conformance suite**; each impl runs it.
- Changing a contract = all 3 agree + ADR line (`docs/team.md`).

## Shared types (`wb-core`)

- `PeerId` — per-run random peer identifier (ADR 0006).
- `BoardId` — board identifier: 8 random bytes (ADR 0025 encodes it).
- `OpId` — operation identifier composed of a peer and its sequence number (ADR 0009); `StrokeId` aliases it and identifies the Add operation for a stroke.
- `Lamport` — logical timestamp used for draw order (ADR 0008).
- `WorldPos` — x/y coordinates in signed 64-bit units; one unit is 1/16 px at zoom 1 (ADR 0014).
- `Aabb` — minimum and maximum `WorldPos` bounds.
- `EncodedPoints` — opaque codec bytes containing an i64 origin and packed deltas.
- `StrokePoints` — decoded origin and signed 32-bit offsets, materialized on demand.
- `Stroke` — id, Lamport timestamp, color, width in world units (ADR 0013), bounding box, and encoded points.
- `Op` — either `Add(Stroke)` or `Erase { id, target }`; new variants may be added (ADR 0015).
- `Time` — injected millisecond time; drivers provide it and it is never read from the OS.
- `Outgoing` — sans-IO destination peer and message.

Wire types (`wb-net`, they hold its `SyncMsg` and `DiscoveryMsg`):

- `Packet` — wire magic, version, and body.
- `Body` — either a discovery message or a board id paired with a `BoardMsg`.
- `BoardMsg` — either an operation or a sync message.

Wire packets use postcard enums. The `WBRD` magic and version are checked before postcard decodes
the rest; version 1 is the initial format and every wire change, including the stroke codec, bumps
it. Only one version is supported at a time, with no negotiation (ADR 0021).

The datagram envelope conformance suite uses injected time to verify separate exact invalid-packet
counters for wrong magic, unknown version, undecodable body, and foreign board id; immediate first
logs per reason; per-reason 10-second throttling and correct “+N more” counts over several intervals;
and version-mismatch logs that include both versions (ADR 0021).

Coordinate rule: every coordinate computation is `i64`; `i32` exists only as decoded offsets.

## File shape

| Section | Contents |
|---|---|
| Operations | named operation, inputs, outputs, and error conditions |
| Data | named shared type and its meaning |
| Guarantees | what callers may rely on |
| Not guaranteed | what callers must not rely on (as important as guarantees) |
| Impls | layer 0 · later layers |
| Conformance tests | one line per test, mapped to a guarantee |

## Status

| Seam | File | Status |
|---|---|---|
| `Board` (type, not a trait) | `board.md` | agreed |
| `StrokeCodec` | `stroke-codec.md` | agreed |
| `SpatialIndex` | `spatial-index.md` | agreed |
| `Camera` (type) | `camera.md` | agreed |
| `StrokeRenderer` | `stroke-renderer.md` | agreed |
| `StrokeCache` | `stroke-cache.md` | agreed |
| `Transport` | `transport.md` | agreed |
| `Discovery` | `discovery.md` | agreed |
| `Broadcast` | `broadcast.md` | agreed |
| `AntiEntropy` | `anti-entropy.md` | agreed |

All agreed at the 2026-10-20 sync (G24); a change needs all 3 + an ADR line.
