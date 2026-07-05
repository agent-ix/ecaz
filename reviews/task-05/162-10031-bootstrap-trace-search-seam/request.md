# Request: Bootstrap Trace Search Seam

Commit: `f6af819`

Summary:
- add a `search::VisibleFrontier::seed_bootstrap_trace(...)` helper that owns conversion from a bootstrap beam trace into visible-frontier and scheduler state
- move the live rescan/bootstrap seeding path in `src/am/scan.rs` behind that search-owned seam
- keep graph-owned layer-0 traversal, scan-owned visited/expanded sets, result adjudication, and the linear fallback unchanged
- add a pure search test covering bootstrap trace truncation, visited marking, scheduler seeding, and entry-source expansion

Please review:
- whether `seed_bootstrap_trace(...)` is the right search-owned seam for rescan bootstrap entry seeding
- whether the scan-side rewiring preserves the current runtime semantics for bootstrap frontier limits and entry-source expanded marking
- whether this is the right smallest live slice before revisiting broader bootstrap fill-policy extraction
