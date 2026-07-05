# Review Request: Pending Scan Output Seam

Commit: `221954d`

Scope:
- `src/am/scan.rs`

Summary:
- move pending heap-tid emission behind `ScanResultState` via a small `PendingScanOutput` seam
- stop having the live runtime path reach into `current().score()` separately from pending-drain
  state when emitting a heap tid
- keep graph/fallback phase producers using the same outer emission point, but with less direct
  scan-owned peeking into result-state internals

Please review:
- whether `take_pending_output()` is the right boundary for the remaining pending-drain shell
- whether the new output seam preserves current-result score semantics while duplicate heap tids are
  drained
- whether this makes the remaining result/drain orchestration in `scan.rs` easier to shrink in
  later A3 slices
