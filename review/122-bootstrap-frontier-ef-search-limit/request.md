# Request: Respect `ef_search` In Bootstrap Frontier Fill

Commit: `3a459a4`

Summary:
- `amrescan` now reads the index reloption `ef_search` into scan-owned state and uses it as the bootstrap frontier limit instead of the old hardcoded 3-slot ceiling.
- Bootstrap seeding, refill-after-consume, and successor admission now all share that scan-owned frontier limit.
- Adds pg coverage that `WITH (ef_search = 1)` keeps the visible bootstrap frontier at one candidate, and pins an older refill test to `WITH (ef_search = 3)` so it continues checking refill behavior rather than default width.

Files:
- `src/am/scan.rs`
- `src/lib.rs`

Why this matters:
- The code already had reloption plumbing for `ef_search`, but scan execution still ignored it and behaved like a fixed-width bootstrap heuristic.
- This slice makes the current bootstrap search breadth configurable through the real index option surface, which is an actual execution step toward a credible graph-search path.
- It also separates one refill-behavior test from the default frontier width so future breadth changes do not create misleading regressions.

Review focus:
- Whether all runtime paths that should honor the configured bootstrap frontier limit now use the scan-owned `ef_search` value
- Whether the new `ef_search = 1` pg test captures the intended visible behavior without depending on incidental entry-point details
- Whether pinning the older refill test to `ef_search = 3` keeps its scope narrow and avoids hiding a real regression
