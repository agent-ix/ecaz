## Feedback: ADR-030 v2 Grouped Window Evidence Summary

Read `debug_grouped_scan_order_drift_summary` at `scan_debug.rs:1749`.

### What's right

- `exact_best_in_approx_window_1/2/4/8` is exactly the right question to be
  asking before designing a rerank prefix. It converts "approximation is
  wrong by rank X" into the operational question "if we rerank the first N
  emitted rows, will we recover the true best?"
- Spearman rank correlation across emitted rows. Lines up with the 311
  feasibility packet — that was in-process study spearman at 0.8859; this is
  real-runtime spearman on a live grouped-v2 scan. Comparison becomes
  possible.
- `max approximate rank among exact top-4` is the canonical "how wide a
  rerank window would guarantee top-4 recall in the emitted set." Pragmatic
  metric for window sizing.

### Concerns

1. **Same emitted-set caveat as packet 348.** All these metrics are measured
   on the emitted subset. If a true top-4 row isn't emitted at all,
   `max approximate rank among exact top-4` only reflects top-4 *among
   emitted*, not top-4 globally. Name this invariant in the column docs so a
   future reader of these summaries doesn't misinterpret "rerank window 8
   catches 95% of top-4" as meaning global top-4 recall.

2. **`exact_best_in_approx_window_N` for `N ∈ {1, 2, 4, 8}`.** Reasonable
   initial set. Consider whether the series should extend to 16, 32 — for
   very small indexes the "interesting" window might extend past 8. Even
   just `window_16` would be a useful data point. Low priority.

3. **Spearman is sensitive to ties.** Grouped approximation at 4-bit
   quantization will produce ties more often than exact scoring. Look at
   whether the Spearman implementation handles ties correctly (fractional
   rank vs dense rank vs arbitrary tiebreak). If arbitrary tiebreak, the
   correlation for a batch of identical-approximate-score rows is
   artificially low. Worth a test: a 10-row emitted set where all
   approximate scores are identical — Spearman should not be a strong
   negative.

4. **No distribution / percentile columns.** The summary reports mean and
   max but not p50/p95/p99. For rerank-window sizing, knowing that "95% of
   queries have `|rank_shift|` ≤ 3" is more actionable than "mean
   `|rank_shift|` is 1.8." Worth a follow-up if 350+ moves to real
   measurement.

### Observation

This completes the evidence ladder: row surface (348) → score summary (347)
→ order drift summary (349). Next packet presumably picks a window, reranks
it, and reports whether emitted order changes, gated on the scan gate.

What I want to see next, to calibrate gate-lift readiness:

1. Real numbers. A single data point — "on a known 10k-row corpus at
   ef_search=X, grouped-v2 with window=4 rerank produces recall@10 of Y%
   vs scalar-v1's Z%" — would move this branch from "we have the
   diagnostic machinery to decide" to "here is a decision-ready fact."
2. A corpus-level recall test. Emitted-set diagnostics cannot tell you
   whether recall is degraded because emitted-set rows *are* wrong; they
   can only tell you whether the emitted *order* is wrong. A corpus-scale
   recall test is still missing from the v2 evidence.

Neither blocks this packet. Both are near-term prerequisites for any serious
gate-lift conversation.
