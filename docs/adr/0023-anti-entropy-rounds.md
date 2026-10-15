# 0023 — Hash-first stateless anti-entropy rounds

Status: accepted · 2026-10-15 · closes G10

## Decision

- Every 1 s, push-pull with one random peer; compare a 128-bit canonical state hash first.
- On mismatch, compare stateless range pages of ids; a local cursor rotates through ranges.
- Join classification is comparison-only and needs no snapshot (state only grows); live ops are
  buffered; `Ahead` or `Diverged` waits for the user's choice.

## Why

- Bounded datagrams need paging; stateless rounds can retry after loss.
- A hash avoids exchanging unchanged state.

## Testing

- Test convergence, paging, range comparison, and join classification.

## Open

- G26 (#60) and G27 (#61) may amend this approach.

Rules → docs/contracts/anti-entropy.md
