# Request: Direct Discovered-Candidate Beam Seeding

Commit: `51f1e4e`

Summary:
- Adds a shared `seed_discovered_candidates` helper in `src/am/scan.rs`.
- Newly discovered bootstrap frontier candidates now enter both the visible frontier vector and the scan-owned beam scheduler at discovery time.
- `top_up_bootstrap_frontier` no longer reseeds the beam by diffing vector tail growth after each refill step.
- Helper-level scan tests now use the same direct-seeding path for newly discovered candidates.

Files:
- `src/am/scan.rs`

Why this matters:
- The previous hybrid contract still relied on the vector frontier as the place where newly discovered candidates first appeared, with the beam catching up afterward by reseeding from appended slots.
- This slice shifts another piece of ownership toward the shared search structure without changing visible tuple-production semantics.
- It reduces duplicated “discover then rediscover” logic and makes later frontier/search extraction easier.

Review focus:
- Whether direct beam seeding at discovery time is the right next ownership shift
- Whether any remaining vector-tail reseeding logic should now be removed entirely
- Whether `seed_discovered_candidates` should move behind a shared search-facing seam in a follow-up
