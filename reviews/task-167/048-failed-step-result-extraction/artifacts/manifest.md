# Task 167 packet 048 artifact manifest

- Head under review:
  - `6d205bdbb6d7087ad1fcc4bff054b2ad990dedaf` — retain primary logs for
    failed DistANN suite steps;
  - `7b20d18fae1bae98d79d76e324224da0780ae735` — structure Task 167 quality,
    calibration, throughput, append, and insert-work metrics;
  - `7e3d3d714c22b57c653c10e34f9acd4f202aa635` — parse the actual hard-gate
    error form emitted by packet 047.
- Owning packet: `reviews/task-167/048-failed-step-result-extraction/`.
- Timestamp: `2026-08-22`.
- Scope: suite report/result extraction only; no PostgreSQL fixture and no
  benchmark rerun.
- Formatter note: `rustfmt` normalized pre-existing indentation in the same
  edited source file. The operator explicitly approved retaining those
  formatter-only edits.

## Focused validation

- Command:
  `cargo test -p ecaz-cli --no-default-features commands::bench::suite::tests::failed_distann_step_retains_primary_log_for_result_extraction -- --exact`.
- Result: passed, 1/1. Artifact: `cargo-test-failed-step.log`, SHA-256
  `ed3036a9bc37f14f174580090928076ee6d81d381f932e0643e8bbf2c253902a`.
- Command:
  `cargo test -p ecaz-cli --no-default-features commands::bench::suite::tests::distann_task167_quality_and_insert_metrics_are_structured -- --exact`.
- Result: passed, 1/1. Artifact: `cargo-test-task167-metrics.log`, SHA-256
  `47ea0260bdaf3d1789dc3818f7cbbe668d0bd0c29791dd82c36b1812ac006e9e`.

## Broader diagnostic

- Command:
  `cargo test -p ecaz-cli --no-default-features commands::bench::suite::tests::`.
- Result: 86 passed, 5 failed, 418 filtered out. The five failures are existing
  command-expansion expectations outside this packet's failed-step extraction
  scope: four seed-variant string expectations and one expected-artifact-list
  expectation.
- Artifact: `cargo-test-suite.log`, SHA-256
  `02c082618a03fa46e42c4293c04a0668ee9e18db9cc79cd67799735fe075e972`.

## Packet 047 report-only proof

- Input: packet 047's original failed-step child log, SHA-256
  `7cc07f1e685666a6a3d5c76984b2ed13ca11f7142f01a48ecaf16a04fe9dea12`.
- Output: packet 047 `results.jsonl`, 15 structured rows, SHA-256
  `80e45d1307a47bc171dce152aefbc608b40ba4a1cc02c91e3b9bb0390aa7e128`.
- Decision row: `physical_benchmark_post_insert_exact_recall`, physical
  `0.848722`, fresh `0.857333`, allowed deficit `0.007000`, delta
  `-0.008611`, `quality_gate_pass=false`.
