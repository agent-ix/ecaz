# Request: Unify Scan Result-State Clearing

Commit: `ac4386e`

Summary:
- make scan result teardown in `src/am/scan.rs` clear both `current_result` and pending duplicate-drain state through one helper
- add an explicit `clear_pending_scan_heaptids` seam and reuse it from:
  - result teardown
  - pending-drain exhaustion
  - pending-drain initialization
- add a unit regression that clearing scan result state wipes the duplicate-drain buffer and counters too

Please review:
- whether `current_result` plus pending heap-TID drain now form a coherent single result-state contract
- whether any runtime path still depends on the old partial-clear behavior
- whether this leaves the staged executor in a cleaner place for a future search-owned result cursor
