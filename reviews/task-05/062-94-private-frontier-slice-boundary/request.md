# Request: Private Frontier Slice Boundary

Commit: `1348872`

Summary:
- Makes the raw visible-frontier slice private to `src/am/scan.rs`.
- Adds an explicit `visible_frontier_snapshot` helper in `src/am/scan.rs`.
- Switches `src/am/scan_debug.rs` to use scan-owned snapshot/slot helpers instead of reading the frontier slice across the module boundary.

Files:
- `src/am/scan.rs`
- `src/am/scan_debug.rs`

Why this matters:
- The earlier visible-frontier seam slices narrowed runtime write/read/iteration behavior inside `scan.rs`.
- `scan_debug.rs` was still the last external consumer of the raw frontier slice, which kept that low-level representation effectively public across modules.
- This slice makes the boundary real: the frontier slice stays private to scan execution, and debug code consumes explicit helper surfaces instead.

Review focus:
- Whether the new snapshot helper is the right narrow debug-facing seam
- Whether any remaining non-scan code still depends on raw frontier-slice access
- Whether the next step should now move remaining Vec-specific removal/materialization behavior behind one stronger container type or start shifting ownership into `search.rs`
