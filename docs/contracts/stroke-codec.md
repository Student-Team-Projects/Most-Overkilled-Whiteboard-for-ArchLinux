# Contract: `StrokeCodec`

Status: draft · How stroke points are shrunk and put into bytes (ADR 0005). Format → `README.md` here.

## Operations

- `simplify` — accepts raw world points and returns their lossy author-side reduction, preserving endpoints and tolerance guarantees. Does not report errors.
- `encode` — accepts points and a byte budget; returns one or more encoded pieces or `CodecError`. Pieces split when required by budget or when an offset cannot fit in i32; neighboring pieces share an endpoint.
- `decode` — accepts encoded points and returns decoded stroke points or `CodecError`; used on demand for drawing and precise hit tests (ADR 0014).

## Guarantees

1. Round trip is exact after simplify: `decode(encode(simplify(raw)))`, expanded to `origin + offset`,
   equals `simplify(raw)`.
2. `simplify` keeps the first and last point, and every raw point lies within `tolerance` of the result.
3. `decode` never panics: any byte input gives `Ok` or `Err`.
4. `encode` returns at least one piece; each is at most `budget` bytes. Decoding and joining pieces
   (dropping the duplicated first point of each neighbour) gives exactly `points`; neighbours
   share one endpoint (G06).

## Not guaranteed

- Exact round trip of points that did not go through `simplify` · a particular compression ratio.
- ⚠️ Codecs are not interchangeable on the wire: the codec is part of the protocol version byte.

## Impls

| Layer | Impl |
|---|---|
| 0 | `Raw`: `simplify` = identity; origin + plain `i32` offsets via postcard |
| later | `Packed`: point reduction → binary delta → bit packing (team's existing code) |

## Conformance tests

| Test | Checks |
|---|---|
| `round_trip` (proptest on random strokes) | 1 |
| `within_tolerance` | 2 |
| `decode_fuzz` (random bytes) | 3 |
| `encode_split_join` (proptest: joined pieces equal original points) | 4 |
| `encode_pieces_fit_budget` (proptest) | 4 |
| bench: bytes/stroke and µs/stroke on generated boards | ratio vs `Raw` |
