# Task 84 Review Request: Configurable Summary Representatives

## Summary

This checkpoint starts the larger Task 84 multi-representative recovery path.
Packet 002 rejected route-prior weighting as a recall recovery policy, and prior
Task 79 evidence showed single-scalar block summaries were exhausted while
multi-representative summaries were the credible next direction.

The code adds a build-time GUC:

- `ec_spire.leaf_block_summary_representatives`
- valid range: `1..8`
- default: `2`, preserving existing RaBitQ summary build behavior

For RaBitQ leaf block summaries, new index builds now store the configured
number of encoded representative payload chunks per block. Existing persisted
indexes remain self-describing through the V4 summary representative count, and
the scan path already scores the max payload chunk for a summary.

## Implementation Notes

- The existing k=2 representative algorithm is preserved as the default path.
- k=1 emits the block mean.
- k>2 uses farthest-first seeding followed by one assignment/mean recompute
  pass.
- The representative count is validated before summary build and by the GUC
  bounds.

## Validation

- `cargo test leaf_block_summaries --no-default-features --features pg18`:
  passed, `2/2`.
- Packet-local log:
  `artifacts/cargo-test-leaf-block-summaries-pg18.log`

## Next Evidence

The next Task 84 packet should build an AWS 1M k=3 summary index under the same
q500 truth lane and compare it against:

- retained `global1152`: `recall@10=0.9832`, `candidate_sum=9,213,846`
- Task 83 blanket-cap controls
- packet 002 route-prior rejection

## Requested Review

Please review the build-time GUC wiring, the default-preserving k=2 behavior,
and whether the k>2 seeding/recompute path is suitable for the AWS k=3
candidate-recovery experiment.

