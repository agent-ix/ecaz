# Task 94 Phase 6 Checkpoint: Suite Block-Kernel Counter Results

## Summary

This checkpoint closes the Task 94 / Task 99 infrastructure gap where `ecaz bench suite` result extraction kept latency table rows but did not preserve direct `[block-kernel-counters]` rows in suite results.

Code checkpoint:

- `efeebf87c` - `Parse block kernel counters in bench suite results`

Artifact checkpoint:

- `bc0e325ab` - `Add Task 94 suite counter artifacts`

## What Changed

- Added parsing for `[block-kernel-counters]` key-value lines in `crates/ecaz-cli/src/commands/bench/suite.rs`.
- Latency artifact extraction now appends those rows to suite results with metric `block_kernel_counters`.
- Existing latency table rows still emit as metric `latency`.
- Direct counter fields are preserved, including `surface`, `quant`, `isa`, `kernel_candidates`, and `scalar_candidates`.
- Added a focused `ecaz-cli` parser test covering a latency table row plus direct block-kernel and legacy Task 87 counter lines.

## Local Validation

Packet-local artifact:

- `artifacts/suite-block-kernel-counter-parser-test.log`

Command:

```text
cargo test -p ecaz-cli latency_result_rows_include_block_kernel_counter_lines
```

Result:

```text
running 1 test
test commands::bench::suite::tests::latency_result_rows_include_block_kernel_counter_lines ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 412 filtered out; finished in 0.00s
```

Formatting:

```text
cargo fmt --check
```

Result: passed, with existing rustfmt warnings about nightly-only import grouping settings.

## Evidence Limits

- This is local parser evidence only. No CI and no AWS/Graviton 4 run was performed.
- The test uses a synthetic latency log fixture; end-to-end suite evidence remains part of the final local/approved-host closeout pass.
