# Feedback: 456-c1-native-build-backlink-score-cache

Reviewed against head `fcfffd0`.

## Verdict

Approved.

## Review focus answers

1. **Is target-local backlink rescoring the right next optimization seam?**

   Yes. `pending` is already sorted by `node_idx` then `layer`, so a target-local
   cache is the natural reuse shape — consecutive rewrites against the same
   target absorb all repeated `score_between(state, target, _)` calls. The seam
   is separate from 455's query-side cache because the query identity is
   different (target vs. new node), so keeping two small scorers is cleaner than
   forcing one generic cache.

2. **Dedicated unit test for the cache reuse seam?**

   Not required. The cache is behavior-preserving by construction — it returns
   exactly the value the prior `metric.score_between(...)` call produced — and
   packet 453's helper tests already pin `add_native_backlinks_*` semantics end
   to end. A seam-specific test would mostly test `HashMap::get`.

## Notes

- Minor: when `needs_new_target_scorer` flips, a fresh `HashMap` is allocated.
  Could instead `.clear()` and update `target_idx` in place to amortize the
  allocation across targets. Micro-optimization, not a blocker — HashMap churn
  is dwarfed by the scoring work this removes.
- The `.chain(once(...)).collect()` was unrolled to `collect` + `push` to avoid
  calling `target_scorer.score(new_node_idx)` inside the chain closure (would
  have conflicted with the borrow already inside `filter_map`). That's the
  right workaround; the resulting shape is still linear and obvious.
- Behavior preservation: all three call sites produce the same
  `ScoredBacklinkNode` set in the same order as before, and
  `select_best_backlink_candidates` is untouched. Confirmed from diff.

## Blockers

None.
