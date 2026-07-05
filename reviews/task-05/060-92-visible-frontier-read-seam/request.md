# Request: Visible Frontier Read Seam

Commit: `665ce49`

Summary:
- Adds one explicit read-side visible-frontier API in `src/am/scan.rs`.
- Moves frontier containment, fallback best-candidate selection, and length checks behind `VisibleCandidateFrontierRef`.
- Keeps the visible frontier Vec in place, but narrows more runtime read authority away from ad hoc slice scans.

Files:
- `src/am/scan.rs`

Why this matters:
- The previous slice introduced a write-side container seam for clear/push/extend/remove.
- Runtime scan logic still open-coded read-side slice walks for containment, fallback head choice, and capacity checks.
- This slice makes the container boundary more real by matching write-side encapsulation with read-side encapsulation, which is the next small step before moving more ownership out of `scan.rs`.

Review focus:
- Whether `VisibleCandidateFrontierRef` is the right narrow read-side surface for the current dual-structure phase
- Whether any important read paths still bypass the local visible-frontier API unnecessarily
- Whether the next step should now be container ownership transfer into `search.rs` or further narrowing of Vec-specific behavior inside `scan.rs`
