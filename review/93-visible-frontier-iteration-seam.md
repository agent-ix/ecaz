# Request: Visible Frontier Iteration Seam

Commit: `19d4ca8`

Summary:
- Extends the local visible-frontier seam in `src/am/scan.rs` to cover iteration and slot reads.
- Moves bootstrap seeding, score-order fallback selection, and candidate slot access onto `VisibleCandidateFrontierRef`.
- Leaves the underlying visible frontier Vec in place, but narrows more runtime read behavior behind the same local boundary.

Files:
- `src/am/scan.rs`

Why this matters:
- The previous two slices established write-side and read-side container seams.
- A few runtime paths still bypassed that boundary by iterating the raw frontier slice directly during bootstrap seeding and fallback score-order scans.
- This slice makes the visible-frontier seam more complete without yet changing ownership or materialization behavior.

Review focus:
- Whether the visible-frontier seam now covers the right remaining runtime read patterns
- Whether any important iteration/slot reads still bypass the local seam unnecessarily
- Whether the next structural move should now be ownership transfer or further narrowing of Vec-specific removal/materialization behavior
