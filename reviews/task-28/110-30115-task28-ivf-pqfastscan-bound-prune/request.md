# Task 28 IVF PQ-FastScan Bound Prune

## Scope

This packet covers the A7 second-attempt code slice in `fa4fea66`: IVF PQ-FastScan scan scoring now uses a per-query suffix max bound to skip grouped-PQ payload scoring once a candidate cannot reach the current pre-rerank frontier.

This is the score-bound pruning lever requested after packet 30078's negative trial. The added state is per prepared query, sized `group_count + 1`, and the running frontier is limited to the existing pre-rerank candidate width.

## Behavior

Before this slice, `materialize_probe_candidates` fully decoded and scored every non-deleted posting tuple that survived selected-list filtering and live-count budgeting.

After this slice:

- `IvfPreparedQuery::PqFastScan` stores a suffix sum of the maximum LUT contribution remaining after each PQ group.
- `score_ip_from_parts_with_min_bound` returns `None` for PQ-FastScan when the partial score plus suffix bound cannot reach the caller's minimum inner-product threshold.
- `materialize_probe_candidates` derives that threshold from the worst retained pre-rerank candidate once the retained set is full.
- Non-PQ-FastScan profiles continue to use the full scoring path.

The pruning path is conservative with deduplication: duplicate-improvement updates are kept in the final dedup map but are not fed into the running frontier, so the threshold can lag but cannot become stricter from duplicate heap TIDs.

## Validation

- `cargo test -p ecaz --lib pq_fastscan`
- `cargo test -p ecaz --lib am::ec_ivf::scan::tests`
- `git diff --check`

The `pq_fastscan` filter also exercised the existing PG18 pg_test cases for PQ-FastScan build, scan, insert, and vacuum paths.

## Next

Run a bounded IVF-only latency/recall smoke on the existing 100k surface to confirm whether the bound cuts scoring work without changing recall. If the smoke is neutral, A7 should be recorded as implemented-but-not-yet-performance-positive rather than blocking further A9/A10 measurement.
