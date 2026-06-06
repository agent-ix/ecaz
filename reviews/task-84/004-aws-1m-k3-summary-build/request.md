# Task 84 Review Request: AWS 1M k=3 Summary Build

## Summary

This packet prepares the first AWS 1M/q500 measurement of the configurable
multi-representative summary build path from packet 003.

The suite builds a separate SPIRE index instead of replacing the retained k=2
surface:

- index: `aws_spire_1m_rabitq_t84_k3_block16_tg256_idx`
- build GUCs:
  - `ec_spire.leaf_block_rows=16`
  - `ec_spire.leaf_block_summary_representatives=3`
- index reloptions otherwise match the retained Task 80/81 block16 tg256
  surface.

The q500 comparison rows are:

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

## Validation

- `ecaz bench suite audit`: passed for
  `suite-aws-1m-k3-summary-build-q500.json` with `7` steps.
- AWS execution pending.

## Requested Review

Please review the suite shape before AWS execution, especially:

- whether the build uses the new k=3 GUC without overwriting retained k=2
  evidence;
- whether the three cap rows are sufficient for the first AWS k=3 readout;
- whether target-block/miss-attribution outputs are enough to compare selected
  leaf recovery against packets 001 and 002.
