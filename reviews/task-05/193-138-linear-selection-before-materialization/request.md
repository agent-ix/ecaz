# Request: Split Linear Selection Before Materialization

Commit: `3ad02da`

Summary:
- change the linear scan path in `src/am/scan.rs` to select a `SelectedScanResult` first, then feed that through the shared result materialization seam
- keep bootstrap and linear paths closer structurally:
  - choose next result candidate
  - materialize into scan-owned result state
  - emit through the shared visible tuple path
- helper coverage now exercises the shared selected-result materialization seam directly

Please review:
- whether `SelectedScanResult` is the right intermediate shape for the current linear path
- whether the page-scan cursor and exhaustion behavior stayed intact across the split
- whether this leaves the executor in a better position for eventually unifying bootstrap and linear result selection under one higher-level contract
