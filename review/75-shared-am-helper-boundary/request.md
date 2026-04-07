# Request: Shared AM Helper Boundary

Commit: `116935f`

Summary:
- Extracts the remaining shared metadata/page/debug utilities out of `src/am/mod.rs` and into `src/am/shared.rs`.
- Moves build-only helper tests into `src/am/build.rs`.
- Leaves `src/am/mod.rs` as a thin module root with constants, module declarations, and narrow reexports.

Files:
- `src/am/build.rs`
- `src/am/graph.rs`
- `src/am/insert.rs`
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/am/shared.rs`
- `src/am/vacuum.rs`

Why this matters:
- This closes the remaining top-level AM shell split, so future work lands in implementation modules instead of growing a second giant catch-all file.
- It creates a clearer distinction between shared storage/metadata utilities and behavior-specific modules like build, insert, scan, and search.

Review focus:
- Whether the new `shared` boundary is the right home for remaining common AM helpers
- Whether any helper still belongs in a behavior-specific module instead of `shared`
- Whether the `mod.rs` root is now thin enough to stay stable as future traversal work continues
