# Task 87 Review Request: TurboQuant Batch Max Scorer

Code commit under review: `40c36f73982459f6fec39590482878445b5b187a`

## Summary

This opens canonical Task 87 on its own branch and lands the first narrow
candidate-batching slice:

- adds `plan/tasks/87-turboquant-candidate-batching.md` and indexes it in
  `plan/tasks/README.md`;
- adds `ProdQuantizer::score_ip_batch_max_lut_no_qjl_4bit`;
- routes SPIRE's zero-gamma contiguous TurboQuant no-QJL 4-bit chunk max path
  through the shared quantizer helper;
- extends the SPIRE quantizer parity test so batch max equals the existing
  per-payload LUT scorer and the existing SPIRE batch scoring path.

The helper intentionally preserves exact per-candidate LUT scoring semantics.
It does not add unsafe code, does not change durable on-disk format, and does
not reland any TQ+ operator-visible format or reloption. The task definition
records TQ+ as a required first-class abstraction concern for future Task 87
work, matching the Task 86 final-audit note.

## Validation

Packet-local artifacts:

- `artifacts/cargo-test-filter-all-targets.log`
- `artifacts/cargo-test-lib-filter.log`
- `artifacts/manifest.md`

Key result lines:

- `test am::ec_spire::quantizer::tests::turboquant_assignment_scorer_uses_no_qjl_4bit_lut_path ... ok`
- `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1989 filtered out; finished in 0.04s`

`cargo fmt --check` and `git diff --check` also passed before the code commit.
`cargo fmt --check` emitted the repository's existing stable-rust warnings about
unstable rustfmt import grouping options.
