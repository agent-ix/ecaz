# Review Request: Task 39 Coverage Baseline And CI Delta Gate

## Summary

This packet completes the Task 39 baseline slice:

- Adds `scripts/check_coverage_delta.sh` and `fixtures/quality/coverage-baseline.tsv`.
- Wires per-PR coverage, weekly mutation, and nightly flake-hunt jobs in GitHub Actions.
- Records baseline coverage for critical modules in `docs/hardening.md`.
- Documents that the current coverage lane is pure-Rust only and does not yet prove live pgrx callback coverage.

## Key Results

- `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/002-coverage-baseline/artifacts/coverage` completed and wrote `coverage/summary.txt` plus `coverage/coverage.json`.
- `coverage/summary.txt` reports total line coverage `0.09%`; the Task 39 critical extension modules in `fixtures/quality/coverage-baseline.tsv` currently record `0.00%`.
- `coverage-delta-check.log` shows every critical baseline path passing the delta rule, for example `quant/prod.rs actual=0.00 baseline=0.00` and `am/common/cost.rs actual=0.00 baseline=0.00`.

## Validation

- `bash -n scripts/hardening.sh scripts/check_coverage_delta.sh`
- `scripts/check_coverage_delta.sh reviews/task-39/002-coverage-baseline/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv`
- `ruby -e "require 'yaml'; YAML.load_file('.github/workflows/ci.yml'); puts 'yaml ok'"`
- `make -n coverage mutants flake-hunt recall-gate cost-gate`

Artifacts are listed in `artifacts/manifest.md`.
