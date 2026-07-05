# Request: Unified Initial Frontier Seeding

Commit: `b281e70`

Summary:
- Makes initial bootstrap frontier seeding in `src/am/scan.rs` use the same direct candidate-to-beam path as discovered refill candidates.
- Introduces a small `seed_existing_frontier_into_expansion` helper for the cases where scan helpers start from a prebuilt visible frontier and an empty scheduler.
- Removes the older `bootstrap_expansion_seed_candidates` scrape path and the associated vector-tail bootstrap logic.

Files:
- `src/am/scan.rs`

Why this matters:
- The prior checkpoint moved discovered candidates into the beam immediately, but initial entry seeding still used a separate vector-scrape path.
- This slice leaves `scan.rs` with one clearer model: visible frontier candidates are either seeded directly into the beam or, if helpers start from a prebuilt frontier, the scheduler is bootstrapped from that existing frontier in one place.
- It reduces the remaining split-brain bootstrap logic before more frontier authority moves behind `src/am/search.rs`.

Review focus:
- Whether the new `seed_existing_frontier_into_expansion` helper is the right residual compatibility seam
- Whether any remaining `scan.rs` helper paths still depend too heavily on visible-vector state
- Whether the next ownership-transfer slice should move visible frontier storage itself behind the shared search structure
