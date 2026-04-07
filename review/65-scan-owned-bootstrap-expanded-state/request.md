# Request: Scan-Owned Bootstrap Expanded State

Commit: `6fc2af6`

Summary:
- Moves bootstrap expanded-source bookkeeping out of a helper-local `Vec<bool>` and into scan-owned state on `TqScanOpaque`.
- Resets expanded-source state on `amrescan`/scan-position reset and frees it during `amendscan`.
- Exposes the expanded-source set through the existing rescan debug helper so pg coverage can assert actual expanded-source ownership.

Files:
- `src/am/scan.rs`
- `src/lib.rs`

Why this matters:
- The bootstrap fill loop now depends on state that should survive helper boundaries and match the scan lifecycle, not a local temporary.
- Later traversal work will need expansion bookkeeping that is owned by scan state, not rebuilt ad hoc per helper call.

Review focus:
- Scan-state ownership and cleanup for `expanded_source_tids`
- Correctness of the score-ordered expansion selector after the helper-local vector removal
- Whether the exposed expanded-source debug view matches the intended bootstrap-fill contract
- Missing edge cases around resets, empty frontiers, or repeated bootstrap fill entry
