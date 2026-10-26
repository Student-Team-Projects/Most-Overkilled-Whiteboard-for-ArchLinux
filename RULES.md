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
| R2 test layout | Tests live in the crate's `tests/` folder and use only the public API. The folder copies `src/`: tests for `src/board/crdt.rs` go in `tests/board/crdt.rs`. Each crate builds one test binary: `tests/main.rs` declares the modules and is the only `.rs` file at the top of `tests/`. Shared helpers, generators and test consts go in `tests/support/`. Seam suites go in `tests/conformance/<seam>.rs` and run against every impl of that seam. Rare exception: a small test of a private item may sit in `#[cfg(test)] mod tests` at the bottom of its file, with a one-line comment saying why it can't go through the public API |
| R3 no magic numbers | runtime and protocol tunables: `src/config.rs` of the owning crate. Test parameters: named consts in `tests/support/config.rs`; a private `mod tests` (R2 exception) keeps its own at its top |
| R4 constructors | a function that builds a `T` is an associated fn (`T::new`, `T::from_…`), never a free fn; adding one to a `wb-core` type needs no ADR |
| documented API | `#![deny(missing_docs)]` in library crates |
| docs without code | `docs/` describe effects, guarantees and data in words; signatures live in code; pseudocode only for algorithms and protocols |
| property tests | `proptest` for CRDT laws and oracle comparisons |
| pedantic lints | `clippy::pedantic` on; every `#[allow]` has a one-line reason |
