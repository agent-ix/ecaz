# Request: Discovered Candidate Search Seam

Commit: `3cd035e`

Summary:
- add a `search::VisibleFrontier::seed_discovered(...)` helper that owns discovered-candidate registration into the visible frontier and scheduler
- move the live bootstrap seeding, single-source refill, and visible-seed top-up paths in `src/am/scan.rs` to call that search-owned seam
- keep graph-owned traversal, scan-owned visited/expanded sets, result adjudication, and the linear fallback unchanged
- add a pure search test covering visited marking, visible-frontier extension, and scheduler seeding order

Please review:
- whether `seed_discovered(...)` is the right search-owned seam for graph traversal outputs entering the runtime frontier
- whether the scan-side rewiring preserves visited marking and scheduler seeding semantics across bootstrap seed, refill, and visible-seed top-up paths
- whether this is the right smallest live slice before attempting any broader bootstrap fill-policy extraction
