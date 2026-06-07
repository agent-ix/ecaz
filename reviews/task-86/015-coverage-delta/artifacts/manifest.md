# Task 86 Packet 015 Artifact Manifest

- head SHA: `55e4928998338042734c411cf817700a6042fb6e`
- task bucket: `reviews/task-86/015-coverage-delta`
- timestamp: `2026-06-07T23:27:24Z`
- lane / fixture / storage format / rerank mode: coverage delta gate; no corpus fixture; no index storage surface; no rerank mode
- table isolation: not applicable

## Artifacts

### `careful-lib-after-page-coverage.log`

- command: `cargo test --manifest-path hardening/careful/Cargo.toml --target-dir target/llvm-cov-target --lib > reviews/task-86/015-coverage-delta/artifacts/careful-lib-after-page-coverage.log 2>&1`
- result: `608 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`

### `make-coverage-after.log`

- command: `make coverage COVERAGE_OUTPUT_DIR=reviews/task-86/015-coverage-delta/artifacts/coverage-after > reviews/task-86/015-coverage-delta/artifacts/make-coverage-after.log 2>&1`
- result: coverage completed and wrote `reviews/task-86/015-coverage-delta/artifacts/coverage-after/summary.txt`
- key result lines:
  - `am/ec_ivf/page.rs ... 96.69%`
  - `quant/prod.rs ... 92.36%`

### `changed-files-after.txt`

- command: `git diff --name-only origin/main HEAD > reviews/task-86/015-coverage-delta/artifacts/changed-files-after.txt`
- result: changed-file list used for the local coverage delta replay

### `coverage-delta-before.log`

- command: `scripts/check_coverage_delta.sh reviews/task-86/015-coverage-delta/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv reviews/task-86/015-coverage-delta/artifacts/changed-files.txt > reviews/task-86/015-coverage-delta/artifacts/coverage-delta-before.log 2>&1`
- result: reproduced the CI failure before adding page codec coverage
- key result lines:
  - `coverage ok: quant/prod.rs actual=92.36 baseline=93.02`
  - `coverage regression: am/ec_ivf/page.rs actual=93.09 baseline=95.86 allowed_drop=2.00`

### `coverage-delta-after.log`

- command: `scripts/check_coverage_delta.sh reviews/task-86/015-coverage-delta/artifacts/coverage-after/summary.txt fixtures/quality/coverage-baseline.tsv reviews/task-86/015-coverage-delta/artifacts/changed-files-after.txt > reviews/task-86/015-coverage-delta/artifacts/coverage-delta-after.log 2>&1`
- result: local delta gate passed after the focused IVF page tests
- key result lines:
  - `coverage ok: quant/prod.rs actual=92.36 baseline=93.02`
  - `coverage ok: am/ec_ivf/page.rs actual=96.69 baseline=95.86`

### `coverage/`

- command: `make coverage COVERAGE_OUTPUT_DIR=reviews/task-86/015-coverage-delta/artifacts/coverage > reviews/task-86/015-coverage-delta/artifacts/make-coverage.log 2>&1`
- result: pre-fix coverage output used to reproduce the delta failure

### `coverage-after/`

- command: `make coverage COVERAGE_OUTPUT_DIR=reviews/task-86/015-coverage-delta/artifacts/coverage-after > reviews/task-86/015-coverage-delta/artifacts/make-coverage-after.log 2>&1`
- result: post-fix coverage output used by `coverage-delta-after.log`
