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
| R2 test layout | All tests go in the crate's `tests/` folder; `src/` has no tests. Each crate has one test binary: its `Cargo.toml` sets `autotests = false` and one `[[test]]` pointing at `tests/main.rs`, which just lists modules, so other files in `tests/` never become extra binaries. `tests/support/` has everything that isn't a test: builders, proptest generators, and named consts in `config.rs`. `tests/conformance/<seam>.rs` has one suite per seam, run on every impl. Other tests: one module per feature, e.g. `tests/board.rs`, which becomes a folder `tests/board/` with files by topic (`undo.rs`, `convergence.rs`) when it grows; a file holds as many tests as its topic needs. Rust compiles `tests/` like an outside crate, so tests call only `pub` items. Rare exception: a small test of a private item may sit in `#[cfg(test)] mod tests` at the bottom of its file, with a one-line comment saying why it can't go through the public API |
| R3 no magic numbers | runtime and protocol tunables: `src/config.rs` of the owning crate. Test parameters: named consts in `tests/support/config.rs`; a private `mod tests` (R2 exception) keeps its own at its top |
| R4 constructors | a function that builds a `T` is an associated fn (`T::new`, `T::from_…`), never a free fn; adding one to a `wb-core` type needs no ADR |
| documented API | `#![deny(missing_docs)]` in library crates |
| docs without code | `docs/` describe effects, guarantees and data in words; signatures live in code; pseudocode only for algorithms and protocols |
| property tests | `proptest` for CRDT laws and oracle comparisons |
| pedantic lints | `clippy::pedantic` on; every `#[allow]` has a one-line reason |
