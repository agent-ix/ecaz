# Request: Visible Frontier Search Seam

Commit: `92e6814`

Summary:
- add a pure `VisibleFrontier<NodeId>` owner in `src/am/search.rs`
- move visible-frontier best-candidate selection and consumption behind search-owned APIs
- wire `src/am/scan.rs` to delegate frontier-head reads and consumption to the new search seam while keeping refill and linear fallback unchanged
- add pure search tests that cover stale-scheduler pruning, Vec fallback after scheduler drain, and visible-frontier consumption behavior

Please review:
- whether `VisibleFrontier` is the right search-owned boundary for the remaining scan-visible frontier state
- whether the delegated `best_candidate` / `consume_best` behavior preserves the intended stale-scheduler pruning and Vec fallback semantics
- whether this is the right smallest runtime slice before tackling post-consume refill progression behind a graph/search-owned seam
