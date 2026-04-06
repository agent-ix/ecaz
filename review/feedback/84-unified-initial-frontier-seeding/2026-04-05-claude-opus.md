# Feedback: Unified Initial Frontier Seeding

Request:
- `review/84-unified-initial-frontier-seeding.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-05

## Response to Review Focus

### Is `seed_existing_frontier_into_expansion` the right residual compatibility seam?

**Yes.** The helper (scan.rs:579-587) exists for one specific scenario: when scan debug/test helpers start from a prebuilt frontier Vec (populated externally) and an empty beam scheduler. It scrapes the existing Vec, filters for `score_valid` and not-yet-expanded candidates, maps to beam candidates, and seeds them via `seed_many`.

This is correct as a compatibility shim because:
1. The normal `amrescan` → `initialize_scan_entry_candidate` path uses `seed_discovered_candidates` (scan.rs:494), which seeds both containers simultaneously
2. Test helpers that manually populate the frontier Vec (e.g., scan.rs:1209) need a way to bring the beam scheduler into sync after external setup
3. `fill_bootstrap_frontier` (scan.rs:589-600) calls this helper after resetting expanded state, ensuring the beam scheduler is initialized from current frontier state

The helper is called from two sites:
- `fill_bootstrap_frontier` (scan.rs:599) — normal initial fill path
- `top_up_bootstrap_frontier` (scan.rs:611-612) — fallback when beam is empty but Vec has candidates

Both are appropriate.

### Do any remaining helper paths depend too heavily on visible-vector state?

**One minor concern.** `top_up_bootstrap_frontier` (scan.rs:603-631) has a fallback at lines 611-612 that reseeds from the Vec if the beam is empty:

```rust
if bootstrap_expansion_mut(opaque).is_empty() {
    seed_existing_frontier_into_expansion(opaque);
}
```

This defensive reseed means the Vec can still be the authoritative source if something goes wrong with beam state. It's a safety net, not a dependency. In the normal flow, the beam is already populated from `seed_discovered_candidates` or from `fill_bootstrap_frontier`'s initial call to `seed_existing_frontier_into_expansion`. The fallback should be removed once the dual-container architecture stabilizes — it hides potential beam-state bugs by silently recovering.

All other paths (`consume_candidate_frontier_head`, `refill_bootstrap_frontier_after_consume`, `materialize_next_bootstrap_frontier_result`) operate on the Vec for data and the beam for ordering, without implicit Vec→beam resyncing.

### Should the next slice move visible frontier storage behind the search structure?

**Not immediately, but it's close.** The current split is:
- **Vec**: holds `ScanCandidate` data (element_tid, source_tid, score, score_valid)
- **BeamSearch**: holds `(score, element_tid)` for ordering — no source_tid or score_valid

To move the Vec behind the search structure, `BeamSearch` would need to carry enough candidate data to replace the Vec entirely. The current `QueuedCandidate<NodeId>` only has `(score, node, sequence)`. Adding `source_tid` and `score_valid` would make it a full candidate container, but that changes the search module's concern from "ordering" to "storage + ordering."

The right next step is probably:
1. Add `source_tid` to the beam candidate (tiny change)
2. Make `BeamSearch::discovery_order` carry full candidate data
3. Replace Vec with beam's `discovery_order` as the data source

This is a 2-3 slice progression, not a single jump.

## Additional Findings

No issues found. The unified seeding path is cleaner than the prior split-brain approach, and `seed_existing_frontier_into_expansion` is the right minimal compatibility shim for test helpers.
