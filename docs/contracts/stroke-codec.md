# Contract: `StrokeCodec`

Status: draft · How stroke points are shrunk and put into bytes (ADR 0005). Format → `README.md` here.

## Signature

```rust
pub trait StrokeCodec {
    /// Lossy, author-side only, once per stroke: reduce points to what will be stored everywhere.
    fn simplify(&self, raw: &[Point]) -> Vec<Point>;
    fn encode(&self, points: &[Point], out: &mut Vec<u8>);
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Point>, CodecError>;
}
```

## Guarantees

1. Round trip is exact after simplify: `decode(encode(simplify(raw))) == simplify(raw)`.
2. `simplify` keeps the first and last point, and every raw point lies within `tolerance` of the result.
3. `decode` never panics: any byte input gives `Ok` or `Err`.

## Not guaranteed

- Exact round trip of points that did not go through `simplify` · a particular compression ratio.
- ⚠️ Codecs are not interchangeable on the wire: the codec is part of the protocol version byte.

## Impls

| Layer | Impl |
|---|---|
| 0 | `Raw`: `simplify` = identity, points via postcard |
| later | `Packed`: point reduction → binary delta → bit packing (team's existing code) |

## Conformance tests

| Test | Checks |
|---|---|
| `round_trip` (proptest on random strokes) | 1 |
| `within_tolerance` | 2 |
| `decode_fuzz` (random bytes) | 3 |
| bench: bytes/stroke and µs/stroke on generated boards | ratio vs `Raw` |
