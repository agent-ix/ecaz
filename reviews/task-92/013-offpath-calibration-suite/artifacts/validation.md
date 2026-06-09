# Validation Log: Task 92 Packet 013

## Formatting

Command:

```sh
cargo fmt
```

Result: completed. The repository rustfmt configuration emitted the existing stable-toolchain warnings for unstable import grouping settings.

## Suite Parser Test

Command:

```sh
cargo test -p ecaz-cli commands::bench::suite::tests::parses_task92_offpath_calibration_config --no-default-features
```

Result:

```text
running 1 test
test commands::bench::suite::tests::parses_task92_offpath_calibration_config ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 411 filtered out; finished in 0.00s
```

## Suite Dry Run

Command:

```sh
cargo run -p ecaz-cli --no-default-features -- bench suite run --config crates/ecaz-cli/suites/task92-offpath-calibration.json --dry-run --manifest-output reviews/task-92/013-offpath-calibration-suite/artifacts/dry-run-suite-manifest.json
```

Result:

```text
[suite:task92-offpath-calibration] wrote reviews/task-92/013-offpath-calibration-suite/artifacts/dry-run-suite-manifest.json
[suite:task92-offpath-calibration] latency-spire-turboquant-lut32-kernel-on -> --database tqvector_bench bench latency --prefix task92_offpath_spire_turboquant --profile ec_spire --k 10 --concurrency 1 --iterations 32 --sweep 32 --bits 4 --seed 42 --force-index --cache-state task92_offpath_kernel_on --task87-candidate-batch-counters --memory-sample-interval-ms 25 --log-output reviews/task-92/013-offpath-calibration-suite/artifacts/latency-spire-turboquant-lut32-kernel-on.log
[suite:task92-offpath-calibration] latency-spire-turboquant-lut32-kernel-off -> --database tqvector_bench bench latency --prefix task92_offpath_spire_turboquant --profile ec_spire --k 10 --concurrency 1 --iterations 32 --sweep 32 --bits 4 --seed 42 --force-index --cache-state task92_offpath_kernel_off --session-guc ec_spire.candidate_batch_scoring=off --task87-candidate-batch-counters --memory-sample-interval-ms 25 --log-output reviews/task-92/013-offpath-calibration-suite/artifacts/latency-spire-turboquant-lut32-kernel-off.log
```

The dry-run compile emitted the existing `LoadedDistributedPlacementConfig::path` dead-code warning from `ecaz-cli`.

## Diff Check

Command:

```sh
git diff --check
```

Result: passed.
