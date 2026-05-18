# Artifact Manifest: Task 39 Coverage Baseline

- Head SHA: `172a5f342f26`
- Task bucket: `reviews/task-39/002-coverage-baseline`
- Timestamp: `2026-05-18T18:39:06Z`
- Lane: Task 39 test-quality coverage baseline and delta gate

## Artifacts

| Artifact | Command | Key result |
| --- | --- | --- |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/002-coverage-baseline/artifacts/coverage` | Coverage completed; total line coverage `0.09%`. Critical extension modules in `fixtures/quality/coverage-baseline.tsv` are recorded at `0.00%` because this lane currently exercises `ecaz-cli` plus `hardening/careful`, not live pgrx callback coverage. |
| `coverage/coverage.json` | Same coverage run | Machine-readable cargo-llvm-cov output for the baseline packet. |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh reviews/task-39/002-coverage-baseline/artifacts/coverage/summary.txt fixtures/quality/coverage-baseline.tsv` | All listed critical-module paths passed the 2 percentage point delta rule. |
| `make-n-quality-and-gates.log` | `make -n coverage mutants flake-hunt recall-gate cost-gate` | Make entrypoints expand without syntax errors after wiring Task 39 and Task 47 gates. |

## Notes

- Isolated one-index-per-table: not applicable to coverage.
- Storage format: not applicable.
- Rerank mode: not applicable.
- The `0.00%` critical-module baseline is intentionally recorded as a gap, not a target.
