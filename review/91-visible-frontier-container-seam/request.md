# Request: Visible Frontier Container Seam

Commit: `0217f33`

Summary:
- Adds one explicit local visible-frontier container seam in `src/am/scan.rs`.
- Moves frontier clear/len/push/extend/remove behavior behind `VisibleCandidateFrontier`.
- Switches runtime scan paths and local unit tests onto that seam instead of open-coding raw `Vec<ScanCandidate>` mutation.

Files:
- `src/am/scan.rs`

Why this matters:
- The recent slices already removed cached frontier-head state, moved head/consume behavior to candidate TID semantics, and localized node-to-index lookup.
- The remaining container ownership problem was that `scan.rs` still mutated the raw `Vec<ScanCandidate>` directly in many places.
- Making the visible frontier a named local boundary is the smallest structural step before moving more ownership out of `scan.rs` and behind shared search/container abstractions.

Review focus:
- Whether this container seam is the right narrow boundary before any deeper `scan`/`search` ownership transfer
- Whether the chosen methods (`clear`, `len`, `push`, `extend`, `remove_node`) are the right minimum surface
- Whether any remaining scan paths still manipulate the visible frontier too directly instead of going through the new seam
