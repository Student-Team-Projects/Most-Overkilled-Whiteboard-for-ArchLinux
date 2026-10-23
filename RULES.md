# Code rules

Every code rule for this repo; code that meets all of them is **kosher**.
Workflow and definition of done → `docs/team.md` · agent map → `AGENTS.md`.

| Rule | Detail |
|---|---|
| understood, not just passing | every line is reviewed and understood by its stakeholder; no AI slop — if you can't explain it, it doesn't merge |
| lean | the smallest code that keeps the contract; no speculative abstraction, no drive-by refactors |
| seam boundaries | as `docs/contracts/README.md` → Rules states |
| no panics in libs | no `unwrap`/`expect`/`panic!` outside tests and `wb-app` `main`; errors are `thiserror` enums |
| R1 self-documenting | rustdoc says in plain words what an item does: guarantees, preconditions, errors; readable without `docs/`. An ADR or contract link goes only at the end, as "See also" |
| R2 test layout | unit tests: `#[cfg(test)] mod tests` at the bottom of the tested file. Per crate: seam suites in `tests/conformance/<seam>.rs`, helpers and generators in `tests/common/`, and one runner `tests/<seam>.rs` that runs the suite on every impl of that seam. Suites use the public API only; a test that needs private items stays in `mod tests`, with a one-line reason |
| R3 no magic numbers | runtime and protocol tunables: `src/config.rs` of the owning crate. Test parameters: named consts at the top of their `mod tests`; those shared across `tests/` files go in `tests/common/config.rs` |
| R4 constructors | a function that builds a `T` is an associated fn (`T::new`, `T::from_…`), never a free fn; adding one to a `wb-core` type needs no ADR |
| documented API | `#![deny(missing_docs)]` in library crates |
| docs without code | `docs/` describe effects, guarantees and data in words; signatures live in code; pseudocode only for algorithms and protocols |
| property tests | `proptest` for CRDT laws and oracle comparisons |
| pedantic lints | `clippy::pedantic` on; every `#[allow]` has a one-line reason |
