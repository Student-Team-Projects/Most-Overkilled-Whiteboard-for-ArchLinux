# Contract: `StrokeCache`

Status: draft · Unpacked points for the strokes the screen needs (ADR 0014, 0018). Lives in
`wb-render`. Format → `README.md` here.

## Operations

- `get` — accepts a board, codec, and stroke id; returns decoded points or `CodecError` if decoding fails.
- `set_resident` — accepts the chunks currently on screen plus margin; points for other strokes may be dropped. Does not report errors.
- `forget` — accepts an erased stroke id and drops its cached points. Does not report errors.

## Guarantees

1. `get(id)` always equals `codec.decode(stroke(id).points)` exactly — the cache never changes
   what is drawn, only how fast.
2. After `forget(id)`, the cache holds nothing for `id`.
3. Memory stays under the configured byte budget (impls with eviction).

## Not guaranteed

- That a `get` is cheap (a miss decodes) · that dropped points stay dropped (a later `get` refills).

## Impls

| Layer | Impl |
|---|---|
| 0 | `DecodeAll`: unpack once per stroke, keep all, `set_resident` is a no-op — **the oracle** |
| M3 | `ChunkWorkingSet`: strokes in resident chunks + 1 ring of margin; drop with the chunk; byte budget |

## Conformance tests

| Test | Checks |
|---|---|
| `matches_decode` (proptest: random gets / residency changes vs `DecodeAll`) | 1 |
| `forget_drops` | 2 |
| `stays_under_budget` while panning a generated board | 3 |
| bench: bytes held and decode count while panning, vs `DecodeAll` | win |
