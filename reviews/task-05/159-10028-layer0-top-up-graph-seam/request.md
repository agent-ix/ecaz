# Request: Layer-0 Top-Up Graph Seam

Commit: `f77f7d7`

Summary:
- add graph-owned helpers for single-source refill successors and visible-seed top-up expansion in `src/am/graph.rs`
- move the remaining scan-owned direct `run_layer0_beam_search(...)` top-up/refill call patterns behind those graph APIs
- keep scan-owned selection, visited/expanded bookkeeping, and the linear fallback unchanged
- add pure graph tests for best-first refill successor ordering and visible-seed expansion output shaping

Please review:
- whether `load_layer0_refill_successors` and `expand_layer0_visible_seeds` are the right graph-owned seams for the current bootstrap top-up path
- whether the new graph helpers preserve the existing runtime semantics for expanded-source marking and discovered-candidate seeding
- whether this is the right smallest runtime slice before extracting more of the remaining scan-side bootstrap fill policy itself
