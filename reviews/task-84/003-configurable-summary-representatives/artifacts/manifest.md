# Task 84 Configurable Summary Representatives Manifest

- Task: `plan/tasks/84-spire-1m-recall-recovery-without-candidate-inflation.md`
- Packet: `reviews/task-84/003-configurable-summary-representatives/`
- Code commit under review: `83dcf40c6`
- Branch: `task-84-spire-recall-recovery`

## Code Scope

- Adds build-time GUC `ec_spire.leaf_block_summary_representatives`.
- Defaults to `2`, matching the existing RaBitQ two-representative summary
  behavior.
- Validates representative counts in `1..8`.
- Preserves the existing k=2 algorithm as the default.
- Adds k=1 block-mean summaries and k>2 farthest-first plus one recompute pass.
- Leaves scan scoring unchanged; existing scan scoring already takes max over
  representative payload chunks in a summary.

## Validation Artifacts

- `cargo-test-leaf-block-summaries-pg18.log`
  - Command: `cargo test leaf_block_summaries --no-default-features --features pg18`
  - Result: `2 passed; 0 failed`

## Next Evidence

This code enables a real Task 84 AWS k=3 build/rebuild packet. That packet
should measure q500 recall/candidate/latency against the retained `global1152`
baseline and Task 83 cap controls before any policy is accepted.

