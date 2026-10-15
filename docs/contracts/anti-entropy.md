# Contract: `AntiEntropy`

Status: draft · The guarantee of convergence (ADR 0003): heals whatever `Broadcast` lost, and powers
late join and the join check (ADR 0006). Sans-IO. Format → `README.md` here.

## Operations

- `tick` — accepts board, peers, and injected current time; returns outgoing sync messages. Does not report errors.
- `on_receive` — accepts sender, sync message, board, and injected time; returns outgoing replies and operations to apply locally. Does not report errors.
- `relation` — accepts board and remote summary; returns `Equal`, `Behind`, `Ahead`, or `Diverged`. Does not report errors.

## Guarantees

1. **Convergence:** if no new ops are made and every pair of peers is eventually connected
   (loss < 100 %, partitions heal), all boards become equal in finite time.
2. `relation` is correct: `Behind` iff local ⊂ remote, `Diverged` iff both have something the other lacks.
3. Every message fits in `Transport::max_datagram()` — large states are sent in pages.
4. Every 1 s, one random peer is selected for a push-pull round. The round compares state hashes
   first; on mismatch, stateless range pages list local op ids in `[lo, hi)`. The receiver answers
   that page alone, pushing the ops whose ids the sender lacks and requesting ids it lacks. A lost page is retried
   next round; rounds keep no page state.
5. The state hash is `Board::state_hash`: its 128-bit hash includes every op id, including adds
   and erases; each inserted op updates it in O(1), duplicates leave it unchanged, and equal boards
   have equal hashes.
6. A page's id list fits within `max_datagram()` less the serialized size of an empty Sync page
   message. Measure that size by serializing the empty message, including packet envelope,
   `Body`/`BoardMsg`/`SyncMsg` discriminants, range bounds, and length prefix; do not hand-count it.
   No page-round state is retained.

## Not guaranteed

- Bandwidth or time bounds · progress during a partition (only after it heals).
- A whole-board hash matching during live drawing; range hashes may replace this approach (G27).

## Impls

| Layer | Impl |
|---|---|
| 0 | `FullSet`: exchange full op-id lists, paged |
| later | `VersionVector`: per-author highest contiguous `seq` + gap lists (ADR 0009); v1 grows by one
  author per run, accepted pending simulation measurement (ADR 0022) |

## Conformance tests

| Test | Checks |
|---|---|
| `converges_under_chaos` (sim, seeded loss/latency/partition, N = 3…100) | 1 |
| `relation_matches_set_compare` (proptest vs brute-force set diff) | 2 |
| `pages_fit_datagram` on 100k-stroke boards | 3 |
| `state_hash_xor_matches_ids` (unit + proptest) | 5 |
| `range_split_and_page_answer` (unit + proptest) | 4, 6 |
| bench: bytes and rounds to converge | win vs `FullSet` |

G26 (#60, bulk catch-up after rejoin) and G27 (#61, range hashes when whole-board hashes rarely
match during live drawing) may amend these rounds.
