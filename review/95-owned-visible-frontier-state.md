# Request: Owned Visible Frontier State

Commit: `4b1ad1c`

Summary:
- Replaces the raw `*mut Vec<ScanCandidate>` in `TqScanOpaque` with an explicit owned visible-frontier state type in `src/am/scan.rs`.
- Keeps the existing visible-frontier helper surface intact while removing the last raw Vec pointer from scan-owned runtime state.

Files:
- `src/am/scan.rs`

Why this matters:
- The earlier slices built helper seams around the visible frontier, but scan state still stored the frontier itself as a raw `Vec` pointer.
- That left the runtime representation unnecessarily low-level even after the slice/write/debug boundaries had been cleaned up.
- This slice makes the frontier a real owned container concept in scan state, which is the first concrete step beyond seam-wrapping toward a stronger container boundary.

Review focus:
- Whether the owned frontier-state type is the right narrow replacement for the raw `Vec` pointer
- Whether any lifecycle or pointer-management assumptions in `amrescan`/`amendscan` changed unintentionally
- Whether the next move should be richer behavior on this owned frontier type or shifting more authority into `search.rs`
