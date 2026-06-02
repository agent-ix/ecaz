# Task 79 Review Request: Summary-Prior Sampled Global Block Scoring

## Summary

This implements the next reviewer-recommended Task 79 slice after packets 017 and 018: keep summary score as the prior, and let sampled rows adjust it instead of replacing it.

The sampled global block path now computes final block score as an upward-only boost from summary score toward the best sampled row score. A bad or missing sample cannot lower a high-summary block. This directly addresses packet 017's failure mode, where noisy one/two-row samples discarded useful summary-ranked blocks.

## Code Changes

- Adds `ec_spire.leaf_block_pruning_sample_summary_prior_weight`, default `0.8`, range `[0.0, 1.0]`.
- Changes sampled global block selection to preserve summary score as a floor:
  - sample lower than summary: keep summary score
  - sample higher than summary: raise score by `(1 - prior_weight) * (sample - summary)`
- Wires the new GUC through the validated snapshot global-pruning path.
- Updates focused tests so the sampled path proves both:
  - bad samples do not demote summary-ranked blocks
  - strong samples can still promote a lower-summary block within the probed frontier

## Validation

Packet-local logs:

- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-test-leaf-block.log`
- `artifacts/manifest.md`

Results:

| command | result |
| --- | --- |
| `cargo fmt --check` | passed |
| `cargo test -p ecaz leaf_block` | 6 passed, 0 failed |

## Next Step

This is not yet proof of a latency win. The next packet should benchmark the new scoring path on the Task 79 RaBitQ surface, starting with final budgets near the candidate gate and prior weights around `0.7`, `0.8`, and `0.9`.
