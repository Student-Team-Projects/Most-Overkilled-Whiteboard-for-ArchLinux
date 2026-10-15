# Contract: `AntiEntropy`

Status: draft · The guarantee of convergence (ADR 0003): heals whatever `Broadcast` lost, and powers
late join and the join check (ADR 0006). Sans-IO. Format → `README.md` here.

## Operations

- `tick` — accepts board, peers, and injected current time; returns outgoing sync messages. Does not report errors.
- `on_receive` — accepts sender, sync message, board, and injected time; returns outgoing replies and operations to apply locally, except in comparison-only join mode where it returns no operations to apply. Does not report errors.
- `relation` — accepts board and remote summary; returns `Equal`, `Behind`, `Ahead`, or `Diverged`. Does not report errors.

## Guarantees

1. **Convergence:** if no new ops are made and every pair of peers is eventually connected
   (loss < 100 %, partitions heal), all boards become equal in finite time.
2. `relation` is correct: `Behind` iff local ⊂ remote, `Diverged` iff both have something the other lacks.
3. Every message fits in `Transport::max_datagram()` — large states are sent in pages.
4. Every 1 s, one random peer is selected for a push-pull round. On mismatch, range pages compare
   stroke ids and tombstone target ids in `[lo, hi)`. Each peer rotates a local cursor through the
   ranges, one per round and wrapping; this cursor is not shared round state. A lost page is retried
   next round.
5. `Board::state_hash` hashes canonical state: all stroke ids ever added plus the set of tombstoned
   target ids, excluding erase OpIds. A newly inserted stroke id or newly tombstoned target updates
   it in O(1). Equal canonical state has equal hashes.
6. Pages are filled by serializing each candidate and stopping before it exceeds
   `Transport::max_datagram()` (or by reserving worst-case postcard prefix and id sizes).
7. During join classification, summaries, range hashes, and id pages are comparison-only: no ops
   are pushed or applied until relation is known and the user chooses merge, copy, or discard for
   `Diverged` or `Ahead`. Version-vector fallback pages in a join also use comparison-only mode.

## Not guaranteed

- Bandwidth or time bounds · progress during a partition (only after it heals).
- A whole-board hash matching during live drawing; range hashes may replace this approach (G27).

## Impls

| Layer | Impl |
|---|---|
| 0 | `FullSet`: exchange full stroke-id and tombstone-target-id sets, paged |
| later | `VersionVector`: per-author highest contiguous `seq` + gap lists (ADR 0009); v1 grows by one
  author per run, accepted pending simulation measurement (ADR 0022) |

## Conformance tests

| Test | Checks |
|---|---|
| `converges_under_chaos` (sim, seeded loss/latency/partition, N = 3…100) | 1 |
| `relation_matches_set_compare` (proptest vs brute-force set diff) | 2 |
| `pages_fit_datagram` on 100k-stroke boards | 3 |
| `state_hash_tracks_canonical_state` (unit + proptest, including repeated erase) | 5 |
| `range_split_and_page_answer` (unit + proptest) | 4, 6 |
| `multi_page_converges` (board requiring at least 3 pages) | 1, 4 |
| `join_comparison_only_until_choice` (including version-vector fallback) | 2, 7 |
| bench: bytes and rounds to converge | win vs `FullSet` |

G26 (#60, bulk catch-up after rejoin) and G27 (#61, range hashes when whole-board hashes rarely
match during live drawing) may amend these rounds.
