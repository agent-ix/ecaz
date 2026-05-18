# Request: Split Bootstrap Selection Before Materialization

Commit: `a222522`

Summary:
- change bootstrap candidate adjudication in `src/am/scan.rs` so it selects a `SelectedScanResult` first, then feeds that through the shared materialization seam
- remove the remaining direct bootstrap-specific result materialization shape from the current staged executor
- after this slice, both bootstrap and linear paths are structurally:
  - select next result
  - materialize into scan-owned result state
  - emit through the shared visible tuple path

Please review:
- whether bootstrap candidate adjudication still preserves the intended “refill only after successful materialization” contract
- whether the new bootstrap selection helper leaves any hidden page-read or stale-candidate behavior behind
- whether this meaningfully improves the path toward a unified higher-level result-selection contract
