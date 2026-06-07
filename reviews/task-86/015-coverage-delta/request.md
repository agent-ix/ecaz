# Task 86 Packet 015: Coverage Delta Gate Fix

## Summary

This packet fixes the remaining Task 86 CI coverage delta failure after TQ+ landed. The code change adds focused IVF page codec coverage for metadata, centroid, list directory, posting, and PQ codebook validation branches, plus the missing public metadata offset constant for byte 34 (`quant_bits`).

Before this packet, local replay matched CI:

- `coverage ok: quant/prod.rs actual=92.36 baseline=93.02`
- `coverage regression: am/ec_ivf/page.rs actual=93.09 baseline=95.86 allowed_drop=2.00`

After the focused tests:

- `coverage ok: quant/prod.rs actual=92.36 baseline=93.02`
- `coverage ok: am/ec_ivf/page.rs actual=96.69 baseline=95.86`

## Validation

- `cargo test --manifest-path hardening/careful/Cargo.toml --target-dir target/llvm-cov-target --lib`
  - `608 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`
- `make coverage COVERAGE_OUTPUT_DIR=reviews/task-86/015-coverage-delta/artifacts/coverage-after`
  - completed successfully
- `scripts/check_coverage_delta.sh reviews/task-86/015-coverage-delta/artifacts/coverage-after/summary.txt fixtures/quality/coverage-baseline.tsv reviews/task-86/015-coverage-delta/artifacts/changed-files-after.txt`
  - passed

## Artifacts

- `artifacts/manifest.md`
- `artifacts/careful-lib-after-page-coverage.log`
- `artifacts/make-coverage-after.log`
- `artifacts/coverage-delta-before.log`
- `artifacts/coverage-delta-after.log`
- `artifacts/coverage/`
- `artifacts/coverage-after/`
