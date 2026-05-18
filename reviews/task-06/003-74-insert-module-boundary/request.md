# Request: Insert Module Boundary

Commit: `e55a71c`

Summary:
- Extracts the live `aminsert` path and its insert-only helpers into `src/am/insert.rs`.
- Keeps the slice mechanical: no duplicate semantics, page layout, metadata locking behavior, or WAL flow changed.
- Leaves `src/am/mod.rs` holding only the shared helpers that insert, build, and scan still use.

Files:
- `src/am/insert.rs`
- `src/am/mod.rs`
- `src/am/routine.rs`

Why this matters:
- This creates a separate ownership lane for future live-insert work instead of keeping insert and scan changes interleaved in `mod.rs`.
- It narrows the remaining shared surface in `mod.rs`, which makes later extraction of truly common AM helpers easier to reason about.

Review focus:
- Whether the extracted insert module is behavior-identical to the old `mod.rs` implementation
- Whether the remaining `mod.rs` helper surface is a sensible short-term shared boundary
- Whether the routine wiring now points cleanly at the new insert entrypoint without hidden coupling
