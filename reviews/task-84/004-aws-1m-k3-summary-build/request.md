# Task 84 Review Request: AWS 1M k=3 Summary Build

## Summary

This packet records the first AWS 1M/q500 attempt for the configurable
multi-representative summary build path from packet 003.

The suite builds a separate SPIRE index instead of replacing the retained k=2
surface:

- index: `aws_spire_1m_rabitq_t84_k3_block16_tg256_idx`
- build GUCs:
  - `ec_spire.leaf_block_rows=16`
  - `ec_spire.leaf_block_summary_representatives=3`
- index reloptions otherwise match the retained Task 80/81 block16 tg256
  surface.

The planned q500 comparison rows were:

- `global1024`: lower candidate budget probe.
- `global1152`: direct retained-budget comparison.
- `global1280`: Task 83 blanket-cap control neighborhood.

## Acceptance Question

Does k=3 block-summary scoring recover AWS 1M/q500 recall above the retained
`global1152` baseline `recall@10=0.9832` without recreating broad candidate
inflation?

The key row is `global1152`. It must beat `0.9832` while staying at or below
the retained `candidate_sum=9,213,846`, or any candidate increase must be
materially better than the Task 83 blanket-cap controls.

## Outcome

The k=3 index build succeeded, but the q500 recall comparison could not be
measured from the retained index after subsequent installs/restarts:

- Build succeeded for
  `aws_spire_1m_rabitq_t84_k3_block16_tg256_idx`.
- Index size: `936 MB`.
- Build timing: `total_ms=1713717` (~28.6 min), with
  `draft_ms=1061544`, `heap_scan_ms=601404`, and `top_graph_ms=22674`.
- Query-only reruns failed on the first `global1024` pipeline before producing
  recall/candidate rows.
- Root diagnostic from SSM:
  `ERROR: ec_spire_distributed: relation context could not be loaded`
  with the ADR-069 hint.

This is not evidence that k=3 failed recall. It shows the retained k=3 index
was not queryable by the SPIRE pipeline after the follow-up install/restart
sequence because required distributed relation context was unavailable.

## Validation

- `ecaz bench suite audit`: passed for
  `suite-aws-1m-k3-summary-build-q500.json` with `7` steps.
- `ecaz bench suite audit`: passed for
  `suite-aws-1m-k3-summary-query-only-q500.json` with `4` steps.
- AWS build execution reached the k=3 index build and completed it.
- AWS query-only diagnostic execution reproduced the relation-context failure.
- AWS final status was captured as `paused`.

## Requested Review

Please review the outcome and next-step choice:

- whether the relation-context loss after install/restart should be handled as
  a Task 84 prerequisite before rebuilding k=3;
- whether the accepted next run should rebuild and query in one uninterrupted
  suite using the latest CLI;
- whether the suite runner child-output change is acceptable durable tooling.
