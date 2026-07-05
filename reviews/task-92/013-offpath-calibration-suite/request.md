# Task 92 Review Request: Off-Path Calibration Suite Config

## Summary

This checkpoint adds a checked-in Task 92 suite config for the off-path counter calibration gate.

Changes:

- Adds `crates/ecaz-cli/suites/task92-offpath-calibration.json`.
- Defines paired SPIRE TurboQuant/LUT32 latency cells over the same prefix/profile:
  - `latency-spire-turboquant-lut32-kernel-on`
  - `latency-spire-turboquant-lut32-kernel-off`
- Both cells carry `quant=turboquant`, `isa=scalar`, and `kernel_status=valid` tags.
- Both cells enable `--task87-candidate-batch-counters`, which now emits the Task 92 block-kernel counter surface.
- The kernel-off cell sets `ec_spire.candidate_batch_scoring=off` through suite `session_gucs`.
- Adds a suite parser/manifest unit test to prevent the calibration config from drifting.

This is the runnable calibration harness foundation. It does not claim the final Task 92 ≤1% off-path drift acceptance result; that still requires an actual suite run on the standard prepared corpus and packet-local kernel-on/off counter logs.

## Validation

Packet-local validation summary:

- `artifacts/manifest.md`
- `artifacts/validation.md`
- `artifacts/dry-run-suite-manifest.json`

Commands passed:

- `cargo test -p ecaz-cli commands::bench::suite::tests::parses_task92_offpath_calibration_config --no-default-features`
- `cargo run -p ecaz-cli --no-default-features -- bench suite run --config crates/ecaz-cli/suites/task92-offpath-calibration.json --dry-run --manifest-output reviews/task-92/013-offpath-calibration-suite/artifacts/dry-run-suite-manifest.json`
- `git diff --check`

## Review Focus

- Confirm the paired kernel-on/kernel-off cells are the right Task 92 calibration shape for the Task 87 LUT32/TurboQuant path.
- Confirm the kernel-off cell uses the correct SPIRE candidate-batch disable GUC while preserving counter collection.
