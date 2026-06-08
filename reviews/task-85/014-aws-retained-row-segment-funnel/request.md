# Review Request: AWS Retained Row Segment Funnel Attempt

## Summary

This packet records the first AWS 1M/q500 attempt to run the retained
block16/global1152 funnel after adding row-segment metrics. The run did not
produce a latency/recall result because the retained DB still had the old
extension SQL return signature.

## Outcome

- Resume succeeded.
- Install of `task-85-spire-product-scale-pareto` succeeded with
  `--skip-extension-recreate`.
- Suite audit passed.
- Bench failed in the first pipeline step:
  `ERROR: column "leaf_row_segment_read_count" does not exist`.
- AWS 1M was paused afterward and final EC2 status shows both instances
  `stopped`.
- A rerun after fallback commit `0fd494def` also failed with the same
  missing-column error because the first guard did not inspect the structured
  Postgres DB error message.
- AWS 1M was paused again after the rerun and final EC2 status shows both
  instances `stopped`.
- A structured fallback rerun after `f17af966c` succeeded. Warm repeat:
  `recall@10=0.9876`, `candidate_sum=9,213,846`,
  `heap_rerank_sum=12,500`, `p50=225.805 ms`, `p95=285.171 ms`,
  `p99=296.588 ms`.
- AWS 1M was paused after the successful run and final EC2 status shows both
  instances `stopped`.

## Interpretation

The failure is expected once observed: skipping extension recreation preserves
the benchmark tables, but also preserves the old pgrx table-returning function
signature. The next checkpoint should add a data-preserving compatibility path
before rerunning AWS.

The successful fallback row is valid for retained recall/latency/candidate
comparison, but not for actual selected row-segment byte evidence. The
row-segment columns were originally inserted in the middle of the table return
type, so the retained legacy SQL signature can mislabel subsequent returned
tuple positions after loading the new shared library. The next code checkpoint
must make the new columns append-only before using legacy-signature funnel split
fields for physical-layout decisions.
