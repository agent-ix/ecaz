# Manifest: Task 92 Packet 013 Off-Path Calibration Suite

- Head SHA: `b4884ac5ce701164a04982e9772d5de10e691d77d`
- Task bucket: `reviews/task-92/`
- Packet path: `reviews/task-92/013-offpath-calibration-suite/`
- Lane: bench-suite config and dry-run validation
- Fixture: SPIRE TurboQuant/LUT32 paired kernel-on/kernel-off latency cells
- Storage format: SPIRE TurboQuant
- Rerank mode: latency command profile `ec_spire`, `k=10`, `nprobe=32`
- Isolated surface: suite expansion and manifest generation; actual calibration run remains a closeout follow-up

## Artifacts

- `dry-run-suite-manifest.json`: dry-run manifest with expanded kernel-on/off commands
- `validation.md`: command log summary for this checkpoint

## Commands

- `cargo test -p ecaz-cli commands::bench::suite::tests::parses_task92_offpath_calibration_config --no-default-features`
- `cargo run -p ecaz-cli --no-default-features -- bench suite run --config crates/ecaz-cli/suites/task92-offpath-calibration.json --dry-run --manifest-output reviews/task-92/013-offpath-calibration-suite/artifacts/dry-run-suite-manifest.json`
- `git diff --check`

## Key Results

- Suite parser test: `1 passed; 0 failed`
- Dry-run manifest:
  - `latency-spire-turboquant-lut32-kernel-on`: `kernel_status=valid`, `quant=turboquant`, `isa=scalar`, counter collection enabled
  - `latency-spire-turboquant-lut32-kernel-off`: `kernel_status=valid`, `quant=turboquant`, `isa=scalar`, `ec_spire.candidate_batch_scoring=off`, counter collection enabled
- `git diff --check`: passed
