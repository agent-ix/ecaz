# Review Request: AM Scan Module Boundary

Scope:
- `src/am/mod.rs`
- `src/am/scan.rs`
- `src/am/routine.rs`
- `spec/adr/ADR-012-am-module-boundaries-for-growth.md`

What changed:
- Extracted scan descriptor lifecycle, scan-local state, bootstrap linear scan execution, and scan debug helpers into `src/am/scan.rs`.
- Updated AM routine registration to bind scan callbacks from the new scan module instead of `src/am/mod.rs`.
- Left `src/am/mod.rs` as the shared AM coordination layer plus the current live insert path and low-level helpers.
- Added ADR-012 to record the intended long-horizon module boundary: traversal and ordered-scan growth should continue under `scan`, and future graph-aware insert work should extract into its own module instead of repopulating `mod.rs`.

Review focus:
- Whether `scan.rs` is now the right growth surface for upcoming graph-traversal work
- Whether the remaining `mod.rs` responsibilities are narrow enough for the next several slices
- Whether the ADR captures a sane future split path before graph scan and graph-aware insert land

Questions to answer:
- Is the `scan` module boundary now strong enough that traversal work can proceed there without another near-term top-level refactor?
- Are there any scan-adjacent helpers that still obviously belong in `scan.rs` rather than `mod.rs`?
- Does ADR-012 set the right rule for future growth, especially around eventual `insert` extraction and a possible later `scan/{state,linear,graph,debug}` split?
