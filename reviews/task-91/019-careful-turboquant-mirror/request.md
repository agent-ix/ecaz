# Task 91 Packet 019: careful TurboQuant mirror

## Summary

This packet fixes the PR Test Quality Coverage failure after the DiskANN
TurboQuant search codec rollout.

The failing coverage job was exercising:

```text
careful_diskann_build::tests::turboquant_build_params_use_direct_search_code_without_sidecar_flags
```

The production DiskANN quantizer already accepts
`VAMANA_SEARCH_CODEC_TURBOQUANT` (`kind = 4`) and computes the direct MSE search
code length. The hardening/careful mirror only accepted grouped PQ and RaBitQ,
so the mirrored `BuildParams::search_code_len()` path rejected kind 4 before the
test assertion could run.

## Code Under Review

- `d29bfb7a73b036a80fe0a8cc4fefffbb20794625`
  `Mirror DiskANN TurboQuant in careful harness`

## Change

- Imports `VAMANA_SEARCH_CODEC_TURBOQUANT` into the careful DiskANN quantizer
  mirror.
- Adds the fixed DiskANN TurboQuant bit width (`4`).
- Mirrors production direct MSE payload sizing as
  `dimensions * 4` bits rounded up to bytes.

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --target-dir target/llvm-cov-target --lib careful_diskann_build::tests::turboquant_build_params_use_direct_search_code_without_sidecar_flags`
  - Result: `1 passed; 0 failed; 621 filtered out`
  - Log: `artifacts/focused-careful-turboquant-test.log`
- `git diff --check`
  - Result: passed
  - Log: `artifacts/git-diff-check.log`

## Notes

This is a CI cleanup packet for Task 91. It does not close Task 91 by itself;
aggregate parity/no-regression closeout evidence is still pending.
