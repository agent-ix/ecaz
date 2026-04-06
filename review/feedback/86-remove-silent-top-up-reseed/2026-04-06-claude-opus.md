# Feedback: Remove Silent Top-Up Reseed

Request:
- `review/86-remove-silent-top-up-reseed.md`

**Reviewer:** Claude (Opus)
**Date:** 2026-04-06

## Response to Review Focus

### Is removing the silent reseed the right call now?

**Yes.** The silent Vec-to-beam recovery that was in `top_up_bootstrap_frontier` (flagged in review 84's feedback as tech debt) has been removed. The current `top_up_bootstrap_frontier` (scan.rs:678-701) trusts that the beam scheduler is already populated from:
1. `seed_discovered_candidates` (scan.rs:632) — at candidate discovery time
2. `seed_existing_frontier_into_expansion` (scan.rs:655) — from `fill_bootstrap_frontier` during initial seeding (scan.rs:674)

With both direct discovery seeding (review 83) and persistent scheduler state (review 79) in place, the beam should never be empty when top-up is called in a valid frontier state. If it is empty, that's a real bug that should surface rather than be silently masked.

The removal makes the beam scheduler contract explicit: if you want the beam to know about candidates, you must seed them. No implicit recovery.

### Do any remaining helper/runtime paths depend on implicit Vec-to-beam recovery?

**No.** I verified the call sites:
- `fill_bootstrap_frontier` (scan.rs:664-676): explicitly calls `seed_existing_frontier_into_expansion` at line 674 before `top_up_bootstrap_frontier` at line 675
- `refill_bootstrap_frontier_after_consume` (scan.rs:808-826): calls `top_up_bootstrap_frontier` at line 820, but by this point the consumed candidate's expansion has already seeded new candidates via `refill` → `seed_discovered_candidates`
- Test helpers that set up frontiers manually now explicitly seed the beam (verified via the test updates described in the review)

No path relies on implicit recovery.

### Should the next slice move frontier ownership behind the search seam?

**Yes — the silent reseed was the last implicit coupling in this direction.** With it gone, the remaining Vec-beam synchronization points are all explicit and localized:
- `seed_discovered_candidates` — atomic dual-seed at discovery
- `forget_queued` — explicit beam cleanup on consume
- `scheduler_best_frontier_node` — stale-node purge on head access

These are the right coupling points for the dual-structure phase. The next step should narrow the Vec's surface area (which reviews 87-95 do) rather than adding new synchronization.

## Additional Findings

No issues found. This follows directly from review 84's feedback recommendation.
