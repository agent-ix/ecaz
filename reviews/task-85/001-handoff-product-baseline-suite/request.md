# Task 85 Review Request: Handoff and Product Baseline Suite

## Summary

This packet starts Task 85 as the larger product-scale Pareto program requested
after Task 84. It does not claim a product profile yet. It defines the first
AWS 1M/q500 baseline suite that future latency work must beat at matched or
improved recall.

The Task 84 handoff is explicit:

- Task 84 packet 006 closed recall recovery with no bounded selected-block
  recovery policy.
- Task 84 packet 007 corrected the user-goal framing to latency at retained
  recall and found no Task 84 mechanism beats the warmed retained k2 surface.
- Reviewer feedback accepted packet 007 and set the operational floor at about
  `recall@10=0.9832`, `candidate_sum=9.21M`, warmed p50 about `255 ms`,
  p95 about `315 ms`, p99 about `332 ms`.

## Baseline Method

The checked-in suite:

`reviews/task-85/001-handoff-product-baseline-suite/suite-aws-1m-product-baseline-q500.json`

will run through `ecaz bench suite` and captures:

- precheck rows for host/database/input/index surface and SPIRE GUCs;
- retained Task 79/81 surface twice in order, so Task 85 uses the warmed repeat
  row as the latency floor rather than a cold artifact;
- Task 83 global-cap controls at `1280` and `1536`;
- storage for the retained SPIRE 1M surface.

The suite keeps q500, `nprobe=96`, `rerank_width=25`, the retained k2 block
summary index, and the existing q500 truth cache constant.

## Comparator Policy

This packet cites the current IVF comparator from
`benchmarks/task51-aws-ivf-rabitq-final-gate/`. HNSW and DiskANN 1M comparator
evidence is not complete enough in the current checkout to close Task 85; later
Task 85 packets must either run those rows or explicitly close with that gap
listed.

## Validation

- `ecaz bench suite audit`: passed for the 6-step Task 85 AWS 1M/q500
  product-baseline suite.
- AWS lifecycle attempt:
  - profile `1m` was confirmed paused before the attempt;
  - resume succeeded;
  - branch install did not return a verifiable completion or fresh visible SSM
    invocation during polling;
  - AWS was paused before running the benchmark suite.

No baseline benchmark result is claimed in this packet yet.

## Requested Review

Please review whether this baseline suite and handoff policy are the correct
Task 85 starting point: future work must beat the warmed retained row at matched
or improved recall, not compare against cold pre-Task-79 surfaces or lower
recall configurations.
