# Artifact Manifest

Task bucket: `reviews/task-39/018-ivf-page-coverage`

Code checkpoint: `a2a142fc38f047c3d1f10d2c4990c264609baa52`

Timestamp: 2026-05-18 America/Los_Angeles / 2026-05-19 UTC

Surface: Task 39 IVF page-codec coverage and mutation through the pgrx-free
`hardening/careful` harness.

## Artifacts

| Artifact | Command | Key Result |
| --- | --- | --- |
| `careful-ivf-page-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib careful_ivf_page -- --nocapture` | 21 passed, 0 failed. |
| `coverage/summary.txt` | `make coverage COVERAGE_OUTPUT_DIR=reviews/task-39/018-ivf-page-coverage/artifacts/coverage` | `am/ec_ivf/page.rs`: 95.86% line coverage. |
| `coverage/careful-summary.txt` | same coverage run | careful-only `src/am/ec_ivf/page.rs`: 95.86% line coverage. |
| `coverage/coverage.json` | same coverage run | merged raw cargo-llvm-cov JSON. |
| `coverage/careful-coverage.json` | same coverage run | careful-only raw cargo-llvm-cov JSON. |
| `coverage/root-summary.txt` | same coverage run | root `ecaz-cli` coverage summary. |
| `mutants/page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_ivf/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/018-ivf-page-coverage/artifacts/mutants MUTANTS_JOBS=2` | Initial run: 221 mutants, 41 missed, 143 caught, 37 unviable. |
| `mutants-rerun/page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_ivf/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/018-ivf-page-coverage/artifacts/mutants-rerun MUTANTS_JOBS=2` | Rerun: 221 mutants, 1 missed, 182 caught, 38 unviable. |
| `mutants-final/page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_ivf/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/018-ivf-page-coverage/artifacts/mutants-final MUTANTS_JOBS=2` | Final run: 220 mutants, 0 missed, 182 caught, 38 unviable. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Passed with pre-existing warnings. |
| `coverage-baseline-check.log` | `make coverage-baseline-check` | `coverage baseline complete for 40 critical paths`. |
| `git-diff-check.log` | `git diff --check` | No whitespace errors. |

## Key Lines Cited

```text
am/ec_ivf/page.rs  1328  55  95.86%
```

```text
220 mutants tested in 5m: 182 caught, 38 unviable
```
