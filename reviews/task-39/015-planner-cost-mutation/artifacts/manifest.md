# Artifact Manifest

Task bucket: `reviews/task-39/015-planner-cost-mutation`

Code checkpoint: `263c36de197454dbcefa387ba84200b9943f61cf`

Timestamp: 2026-05-18 America/Los_Angeles / 2026-05-19 UTC

Surface: Task 39 planner-cost coverage and mutation, pgrx-free `hardening/careful` harness for the pure planner-cost model. This is not a live PostgreSQL callback run.

## Artifacts

| Artifact | Command | Key Result |
| --- | --- | --- |
| `careful-cost-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib careful_common_cost` | 13 passed, 0 failed. |
| `make-coverage.log` | `make coverage` | `ecaz-cli`: 355 passed; `hardening/careful`: 219 passed; reports written under `target/quality/coverage`. |
| `coverage-summary.txt` | copied from `target/quality/coverage/summary.txt` after `make coverage` | `am/common/cost.rs`: 98.98% line coverage. |
| `careful-coverage-summary.txt` | copied from `target/quality/coverage/careful-summary.txt` after `make coverage` | `src/am/common/cost.rs`: 98.98% line coverage in careful. |
| `coverage.json` | copied from `target/quality/coverage/coverage.json` after `make coverage` | merged JSON coverage report. |
| `careful-coverage.json` | copied from `target/quality/coverage/careful-coverage.json` after `make coverage` | careful harness JSON coverage report. |
| `cost.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/common/cost.rs MUTANTS_OUTPUT_DIR=reviews/task-39/015-planner-cost-mutation/artifacts MUTANTS_JOBS=2` | Initial run: 61 mutants, 18 missed, 37 caught, 6 unviable. |
| `rerun/cost.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/common/cost.rs MUTANTS_OUTPUT_DIR=reviews/task-39/015-planner-cost-mutation/artifacts/rerun MUTANTS_JOBS=2` | Final run: 58 mutants, 0 missed, 52 caught, 6 unviable. |
| `coverage-delta-check.log` | `scripts/check_coverage_delta.sh target/quality/coverage/summary.txt fixtures/quality/coverage-baseline.tsv` | `am/common/cost.rs` actual 98.98, baseline 98.98. |
| `coverage-baseline-complete.log` | `scripts/check_coverage_baseline_complete.sh` | Baseline complete for 40 critical paths. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Finished successfully. |
| `git-diff-check.log` | `git diff --check` | No whitespace errors. |

## Key Lines Cited

```text
am/common/cost.rs                                                                       282                 3    98.94%          25                 0   100.00%          294                 3    98.98%           0                 0         -
```

```json
"total_mutants": 58,
"missed": 0,
"caught": 52,
"timeout": 0,
"unviable": 6
```
