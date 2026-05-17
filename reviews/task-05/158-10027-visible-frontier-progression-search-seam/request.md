# Request: Visible Frontier Progression Search Seam

Commit: `3c9ef54`

Summary:
- add `VisibleFrontier::advance_after_consume` in `src/am/search.rs`
- move post-consume visible-frontier progression behind the search seam while keeping candidate adjudication in `scan.rs`
- refactor scan-side refill/top-up helpers to operate on the already-borrowed visible frontier and beam scheduler instead of reacquiring scan-owned containers
- add pure search tests for unexpanded-source refill/top-up and already-expanded-source skip behavior

Please review:
- whether `advance_after_consume` is the right search-owned boundary for post-consume visible-frontier progression
- whether the new direct `visible_frontier` / `BeamSearch` refill hooks in `scan.rs` preserve the existing “refill only after successful selection” and expanded-source semantics
- whether this is the right smallest runtime slice before extracting more of the graph-backed top-up path itself
