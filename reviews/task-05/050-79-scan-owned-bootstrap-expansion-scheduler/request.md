# Request: Scan-Owned Bootstrap Expansion Scheduler

Commit: `804693f`

Summary:
- Moves bootstrap expansion scheduling in `src/am/scan.rs` into scan-owned state instead of rebuilding a temporary beam scheduler during each top-up cycle.
- Keeps the current frontier storage, emitted ordering, and visible tuple-production behavior unchanged.

Files:
- `src/am/scan.rs`

Why this matters:
- This is the first step from “search helper used opportunistically” to “search state owned by the executor”.
- It reduces per-cycle scheduler reconstruction and gives later traversal work a stable place to hang longer-lived frontier state.
- It preserves current behavior while making a later move of more frontier ownership out of `scan.rs` much smaller.

Review focus:
- Whether the new scan-owned scheduler lifetime/reset/free behavior is correct across `amrescan`, exhaustion, and `amendscan`
- Whether the scheduler and `expanded_source_tids` can drift out of sync under the current consume/refill rules
- Whether this is the right intermediate step before moving more candidate/frontier state behind the shared search boundary
