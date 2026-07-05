# Request: Unify The Staged Result-Materialization Step

Commit: `8a1fb1e`

Summary:
- `amgettuple` now delegates bootstrap-vs-linear result selection through one shared `materialize_next_scan_result` step.
- Visible tuple emission still happens through the same explicit pending-drain path, so this slice only removes top-level branch duplication around result materialization.
- The staged executor now reads more directly as: drain current result, materialize next result, drain it.

Files:
- `src/am/scan.rs`

Why this matters:
- The last few slices made pending duplicate drain explicit, removed stale fallback drain branches, and made the linear path match the bootstrap path structurally.
- This follow-on keeps that direction going by removing the remaining bootstrap/linear branch duplication in `amgettuple`.
- A simpler staged execution flow should make later work on genuinely ordered beam-driven result production easier to reason about.

Review focus:
- Whether `materialize_next_scan_result` is now the right shared seam for the current staged executor
- Whether the bootstrap-first fallback semantics remain intact after moving the top-level branching behind that helper
- Whether this leaves the scan runtime in a cleaner state for the next ownership transfer or ordering slice
