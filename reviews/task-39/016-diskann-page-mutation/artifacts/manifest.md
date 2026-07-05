# Artifact Manifest

Task bucket: `reviews/task-39/016-diskann-page-mutation`

Code checkpoint: `33e6f6f86d1c6d302db46c53bef83ce6c97050f4`

Timestamp: 2026-05-18 America/Los_Angeles / 2026-05-19 UTC

Surface: Task 39 DiskANN page-codec mutation through the pgrx-free `hardening/careful` harness.

## Artifacts

| Artifact | Command | Key Result |
| --- | --- | --- |
| `careful-diskann-page-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib careful_diskann_page` | 8 passed, 0 failed. |
| `page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_diskann/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/016-diskann-page-mutation/artifacts MUTANTS_JOBS=2` | Initial run: 11 mutants, 2 missed, 7 caught, 2 unviable. |
| `rerun/page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_diskann/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/016-diskann-page-mutation/artifacts/rerun MUTANTS_JOBS=2` | Intermediate run: 11 mutants, 1 missed, 8 caught, 2 unviable. Remaining survivor was equivalent. |
| `final/page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_diskann/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/016-diskann-page-mutation/artifacts/final MUTANTS_JOBS=2` | Final run: 10 mutants, 0 missed, 8 caught, 2 unviable. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Finished successfully. |
| `git-diff-check.log` | `git diff --check` | No whitespace errors. |

## Key Lines Cited

```json
"total_mutants": 10,
"missed": 0,
"caught": 8,
"timeout": 0,
"unviable": 2
```
