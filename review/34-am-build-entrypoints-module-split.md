# Review Request: AM Build Entrypoints Module Split

Scope:
- `src/am/mod.rs`
- `src/am/build.rs`
- `src/am/routine.rs`

What changed:
- Extracted `ambuild`, `ambuildempty`, and the build-side heap scan callback wiring into `src/am/build.rs`.
- Left the deeper build helpers and graph/page staging logic in `src/am/mod.rs` so the slice stays mechanical.
- Updated `src/am/routine.rs` to register the extracted build entrypoints.

Review focus:
- Whether the extracted build entrypoints preserve the exact build behavior and callback wiring
- Whether the helper visibility opened from `mod.rs` to `build.rs` is still narrow and appropriate
- Whether this is a clean intermediate split before moving more substantial build internals

Questions to answer:
- Do the extracted build entrypoints remain behavior-identical to the prior inline implementation?
- Are the newly shared `BuildState` and build callback boundaries exposed narrowly enough?
- Is this the right stopping point before considering a deeper build-helper extraction?
