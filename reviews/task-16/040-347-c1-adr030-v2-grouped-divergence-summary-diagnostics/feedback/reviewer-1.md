## Feedback: ADR-030 v2 Grouped Divergence Summary Diagnostics

Read `debug_grouped_scan_comparison_summary` at `scan_debug.rs:1694`.

### What's right

- Scalar scans are proven to report zero/null grouped metrics. Important for
  tooling that runs the summary across a mixed fleet — scalar indexes will not
  look like they have grouped behavior.
- Emitted / grouped / compared / missing counts are exposed separately. Not a
  single "total" — each is independently observable. An operator seeing
  `compared < grouped` can immediately tell something is dropping comparisons.
- Mean absolute, max absolute, mean signed. Three complementary metrics:
  - mean absolute: typical magnitude of approximation error
  - max absolute: worst-case approximation error
  - mean signed: whether approximation is biased in one direction

  All three are needed for the rerank-window decision. Missing any one would
  leave a real phenomenon unobservable.

### Concerns

1. **`missing comparison count`.** Where does this non-zero value come from
   currently? Per packet 346, a missing comparison fires `pgrx::error!` rather
   than silently skipping. So today `missing == 0` always, or the query
   errors. If the metric is there for a future state where missing is
   tolerated, fine — but then the 346 behavior of erroring on mismatched
   cold payload needs to be reconciled with this metric being advisory.
   Right now these two code paths tell inconsistent stories. See my packet
   346 feedback.

2. **Summary is per-query.** Useful for debugging a single query. For
   gate-lift decisions you want per-corpus summaries across hundreds of
   queries. The summary is additive (counts, means can be recombined with
   standard formulas) — worth documenting that so a batch harness can
   aggregate across queries without rewriting.

3. **No standard-deviation / p95 / p99 score delta.** Means hide
   distribution shape. If 99% of rows have near-zero delta and 1% have
   enormous delta, the rerank-window packet needs to see the distribution.
   Today max-absolute captures the worst case but doesn't tell you how
   often. Consider adding p95/p99 to a future summary packet, or emitting
   a histogram alongside.

### Observation

Useful as one rung of the diagnostic ladder. Not the final step — the mean/
max/signed triple is the right first pass, but the rerank-window decision
will want distribution shape. 348/349 answer part of this with rank-based
diagnostics, but score distribution shape remains a gap.
