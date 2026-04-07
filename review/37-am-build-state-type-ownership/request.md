# Review Request: AM Build State Type Ownership

Scope:
- `src/am/mod.rs`
- `src/am/build.rs`

What changed:
- Moved `BuildState` and `BuildTuple` into `src/am/build.rs`.
- Updated remaining callers and tests to use the build module as the owning namespace for build-only types.
- Left `src/am/mod.rs` with the shared `decode_heap_tid` and scan/live-insert logic, instead of also owning build-state structures.

Review focus:
- Whether the build-only type ownership is now in the right module
- Whether the remaining cross-module references are narrow and coherent
- Whether this leaves the codebase ready to shift attention back toward traversal work instead of more shell refactoring

Questions to answer:
- Do `BuildState` and `BuildTuple` now clearly belong in `src/am/build.rs`?
- Are the remaining shared hooks between `mod.rs` and `build.rs` narrow enough?
- Is the AM build boundary now clean enough to stop modularization and resume serious traversal work?
