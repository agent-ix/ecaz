# Request: Scan Debug Module Boundary

Commit: `4916605`

Summary:
- Extracts the scan debug/test-only surface out of `src/am/scan.rs` and into `src/am/scan_debug.rs`.
- Keeps scan execution, traversal state, and unit-tested runtime helpers in `src/am/scan.rs`.
- Widens only the scan internals needed by the debug module to `pub(super)`.

Files:
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/am/scan_debug.rs`
- `src/lib.rs`

Why this matters:
- This is the remaining structural hotspot blocking parallel work in scan execution.
- It gives traversal/search work a cleaner home in `scan.rs` while isolating the large pg-test/debug surface in a separate file.
- It keeps the debug boundary explicit instead of letting test hooks continue to grow inside the executor.

Review focus:
- Whether the `scan` vs `scan_debug` boundary is clean enough for parallel work
- Whether any newly `pub(super)` scan internals should instead be hidden behind narrower helper functions
- Whether the updated pg regression on bootstrap refill provenance still matches the intended executor contract
