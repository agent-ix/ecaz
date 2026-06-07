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

## Interpretation

The failure is expected once observed: skipping extension recreation preserves
the benchmark tables, but also preserves the old pgrx table-returning function
signature. The next checkpoint should add a data-preserving compatibility path
before rerunning AWS.
