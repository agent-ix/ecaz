# Request: Visible-Seed Top-Up Search Seam

Commit: `c196f2a`

Summary:
- add a `search::VisibleFrontier::top_up_from_visible_seeds(...)` helper that owns visible-seed filtering, remaining-capacity handling, expanded-source marking, and discovered-candidate registration
- move the live post-success visible-seed top-up path in `src/am/scan.rs` behind that search-owned seam
- keep graph-owned layer-0 visible-seed expansion, scan-owned visited/expanded sets, result adjudication, and the linear fallback unchanged
- add a pure search test covering unexpanded-seed selection, capacity propagation, expanded-source marking, and successor seeding

Please review:
- whether `top_up_from_visible_seeds(...)` is the right search-owned seam for the live post-success bootstrap top-up path
- whether the scan-side rewiring preserves current runtime semantics for visible-seed filtering, expanded-source updates, and discovered-successor registration
- whether this is the right smallest runtime slice before tackling any remaining bootstrap fill-policy surface
