# Review Request: Scan Current-Result State

Scope:
- `src/am/mod.rs`
- `src/lib.rs`

What changed:
- `TqScanOpaque` now tracks an explicit current-result tuple pointer and score-valid bit in scan-owned state.
- The bootstrap linear scan populates that current-result slot when it selects a live element tuple, and clears it on rescan and full exhaustion.
- Regression coverage now verifies that the current-result slot is invalid before the first tuple and becomes populated after the first successful `amgettuple`.

Review focus:
- Whether the current-result state boundary is the right next step for ordered traversal groundwork
- Whether clearing semantics across rescan and exhaustion are correct and complete
- Whether the new debug/test surface is narrow enough for this stage

Questions to answer:
- Is tuple-pointer-only result state the right minimal contract before adding score/candidate machinery?
- Are there any stale-state risks left around duplicate draining, rescan, or exhaustion?
- Is there a cleaner place to anchor current-result state than `TqScanOpaque` at this stage?
