# Contract: `StrokeCache`

Status: agreed 2026-10-20 · Unpacked points for the strokes the screen needs (ADR 0014, 0018). Lives in
`wb-app`. Format → `README.md` here.

## Operations

- `get` — accepts a stroke; returns its decoded points, or `CodecError` if decoding fails. A cache
  decodes with the one codec it was built with, and tells strokes apart by id (one id always has the
  same points, as on a board).
- `set_resident` — accepts the world area on screen plus margin; points of strokes outside it may be dropped. Does not report errors.
- `forget` — accepts an erased stroke id and drops its cached points. Does not report errors.

## Guarantees

1. `get(stroke)` always equals `codec.decode(stroke.points)` exactly — the cache never changes
   what is drawn, only how fast.
2. After `forget(id)`, the cache holds nothing for `id`: the next `get` of that stroke decodes it again.
3. Memory stays under the configured byte budget (impls with eviction).

## Not guaranteed

- That a `get` is cheap (a miss decodes) · that dropped points stay dropped (a later `get` refills).

## Impls

| Layer | Impl |
|---|---|
| 0 | `DecodeAll`: unpack once per stroke, keep all, `set_resident` is a no-op — **the oracle** |
| M4 | `ChunkWorkingSet`: strokes in the chunks under the resident area + 1 ring of margin; drop with the chunk; byte budget |

## Conformance tests

| Test | Checks |
|---|---|
| `matches_decode` (proptest: random gets / residency changes vs `DecodeAll`) | 1 |
| `forget_drops` (a get after `forget` decodes again) | 2 |
| `stays_under_budget` while panning a generated board (M4: `DecodeAll` has no budget) | 3 |
| bench: bytes held and decode count while panning, vs `DecodeAll` | win |
