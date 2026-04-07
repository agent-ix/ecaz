# Request: Result-State Owned Bookkeeping

Commit: `f040715`

Summary:
- move remaining scan-result bookkeeping in `src/am/scan.rs` more directly onto `ScanResultState`
- remove free-function wrappers for pending duplicate drain and result clearing
- make shared result materialization delegate to the owned result-state container while preserving emitted-element tracking at the scan boundary

Please review:
- whether any result-state lifecycle behavior still leaks outside `ScanResultState` unnecessarily
- whether emitted-element tracking still happens at the right boundary now that current-result bookkeeping moved further into the owned state container
- whether this meaningfully improves the path toward a search-owned result cursor or other higher-level ordered traversal surface
