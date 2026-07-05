# Request: Refill Source Search Seam

Commit: `971784e`

Summary:
- add a `search::VisibleFrontier::refill_from_source(...)` helper that owns remaining-capacity handling and discovered-successor registration for single-source refill
- move the live post-success single-source refill path in `src/am/scan.rs` behind that search-owned seam
- keep graph-owned layer-0 successor loading, scan-owned visited tracking, result adjudication, and the linear fallback unchanged
- add a pure search test covering remaining-capacity propagation, successor seeding, and visited marking

Please review:
- whether `refill_from_source(...)` is the right search-owned seam for the live post-success refill path
- whether the scan-side rewiring preserves current runtime semantics for successor loading capacity and discovered-successor registration
- whether this is the right smallest runtime slice before deciding if any remaining bootstrap helper surface is still worth extracting
