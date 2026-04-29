# Task 28 IVF Scan Volume Counters

## Scope

Code checkpoint for Task 28 IVF follow-up work. This packet records the new scan-volume EXPLAIN counters added in commit `ebe12217`.

## Change

The IVF EXPLAIN counter surface now reports additional score-volume detail during candidate materialization:

- `Postings Visited`: selected, live posting tuples reached by the scan.
- `Postings Scored`: visited postings that completed quantized scoring.
- `Postings Pruned By Bound`: visited postings skipped by the current top-k score bound.
- `Heap TIDs Scored`: heap TIDs attached to postings that completed scoring.
- `Candidates Inserted`: unique heap TID candidates inserted into the dedup map.

The existing `Candidates Scored` counter is unchanged and continues to count candidate heap TIDs considered after posting scoring.

## Validation

Run on commit `ebe12217`:

- `cargo test ivf_explain --no-default-features --features pg18`
  - Result: passed, 2 IVF EXPLAIN unit tests.
- `cargo pgrx test pg18 test_pg18_explain_option_emits_ecaz_stats_group_for_ec_ivf`
  - Result: passed, 1 PG18 pgrx IVF EXPLAIN smoke test.
- `git diff --check`
  - Result: passed.

## Notes

This packet makes no performance measurement claim. The purpose is to make the next IVF sweep explain latency by posting and candidate volume rather than relying only on recall/latency aggregates.
