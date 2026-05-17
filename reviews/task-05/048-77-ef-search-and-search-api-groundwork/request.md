# Request: ef_search And Search API Groundwork

Commit: `03106c8`

Summary:
- Adds `ef_search` reloption groundwork to the AM options surface without changing planner or executor behavior yet.
- Extends `src/am/search.rs` with incremental beam-search APIs (`seed_many`, `peek_best`, `expand_one`, frontier inspection).
- Uses that shared search seam in `src/am/scan.rs` for score-ordered bootstrap expansion source selection.

Files:
- `src/am/build.rs`
- `src/am/mod.rs`
- `src/am/options.rs`
- `src/am/scan.rs`
- `src/am/search.rs`

Why this matters:
- It gives parallel scan work a stable pure-Rust search helper instead of growing more ordering logic directly in `scan.rs`.
- It lands the `ef_search` knob early so later scan wiring does not need another reloptions pass.
- It is the first real executor-side use of the new `search` module, while keeping visible scan behavior unchanged.

Review focus:
- Whether the incremental `search.rs` API is minimal but sufficient for upcoming scan integration
- Whether `ef_search` defaults and bounds are reasonable groundwork for later executor/planner wiring
- Whether using the shared search helper for bootstrap expansion arbitration is the right first integration seam
