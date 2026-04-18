## Feedback: ADR-030 v2 Grouped Order Drift Diagnostics

Read `debug_grouped_scan_comparison_rows` at `scan_debug.rs:1650` and the shared
`debug_scan_uses_grouped_storage` helper at `scan_debug.rs:1635`.

### What's right

- `debug_scan_uses_grouped_storage` centralizes the grouped-format check.
  Packets 347/348/349 all route through it. One place to update if storage
  format detection ever changes.
- Row surface reports approximate rank, exact rank, and signed rank shift.
  Rank-based metrics are robust to the score-sign convention (a simple sign
  flip would destroy score-delta metrics but leave rank metrics intact).
- Scalar scans keep grouped-only columns `NULL` rather than zero. That's the
  right SQL shape: `NULL` means "not applicable" and cannot be aggregated as
  if it were zero. `0` would lie if you `AVG` across a mixed fleet.

### Concerns

1. **`exact_rank` is rank among *emitted* rows, not absolute rank.** The
   packet text says "sorts grouped exact comparison scores to derive a 1-based
   exact rank." That rank is over the subset of rows this query emitted, not
   over all rows in the index. That's a meaningful distinction:

   - If approximate-scan emitted the top-100, and the true top-10 is within
     those 100, `exact_rank` 1-10 in the emitted set is the true top-10.
     Good.
   - If approximate-scan missed a true top-10 row entirely, that row never
     appears in this diagnostic. The rank shift is measured over "rows we
     saw," not "rows we should have seen."

   This is fine for rerank-window sizing (you only rerank what you emit), but
   it means this diagnostic cannot detect recall holes — only reordering
   within the emitted set. Worth naming this invariant explicitly in the
   summary column docstring, because it is the *exact* question a reader will
   ask.

2. **Rank-shift semantics are bounded by emitted count.** For a query that
   emits N rows, the max possible `|exact_rank_shift|` is N-1. So Spearman
   correlation reported in packet 349 is Spearman on the emitted subset — an
   index that emits 10 "right-enough" rows but misses the top 5 will show
   Spearman ≈ 1.0 while having recall 50%. Again, measuring within the
   emitted set, not across the corpus.

3. **Signed vs unsigned rank shift.** Signed shift is useful for detecting
   systematic bias (approximation consistently rates rows higher or lower
   than exact). Good metric. But the summary in packet 349 reports
   `mean_abs_rank_shift` and `max_abs_rank_shift` but not mean signed. A
   systematic upward or downward bias would be invisible in 349's summary.
   Consider adding.

### Observation

Rank-based diagnostics complement score-based diagnostics. Together with the
score metrics from 347, this lets you see both "how wrong are the numbers"
(score delta) and "how wrong is the order" (rank drift). Right next rung.
