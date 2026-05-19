# Artifact Manifest

Task bucket: `reviews/task-39/017-hnsw-page-mutation`

Code checkpoint: `59c81555121c12c8c72daba4299587bf4253dd04`

Timestamp: 2026-05-18 America/Los_Angeles / 2026-05-19 UTC

Surface: Task 39 HNSW page-codec mutation through the pgrx-free `hardening/careful` harness.

## Artifacts

| Artifact | Command | Key Result |
| --- | --- | --- |
| `careful-hnsw-page-tests.log` | `cargo test --manifest-path hardening/careful/Cargo.toml --lib hnsw_page -- --nocapture` | 42 passed, 0 failed. |
| `page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_hnsw/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/017-hnsw-page-mutation/artifacts MUTANTS_JOBS=2` | Initial run: 477 mutants, 119 missed, 283 caught, 75 unviable. |
| `rerun-1/page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_hnsw/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/017-hnsw-page-mutation/artifacts/rerun-1 MUTANTS_JOBS=2` | Intermediate run: 476 mutants, 9 missed, 386 caught, 6 timeouts, 75 unviable. |
| `rerun-2/page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_hnsw/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/017-hnsw-page-mutation/artifacts/rerun-2 MUTANTS_JOBS=2` | Intermediate run: 444 mutants, 1 missed, 369 caught, 74 unviable. |
| `final/page.rs.mutants/mutants.out/*` | `make mutants MUTANTS_MODULE=src/am/ec_hnsw/page.rs MUTANTS_OUTPUT_DIR=reviews/task-39/017-hnsw-page-mutation/artifacts/final MUTANTS_JOBS=2` | Final run: 444 mutants, 0 missed, 370 caught, 74 unviable. |
| `cargo-check-pg18-bench.log` | `cargo check --all-targets --no-default-features --features pg18,bench` | Finished successfully with pre-existing warnings. |
| `git-diff-check.log` | `git diff --check` | No whitespace errors. |

## Key Lines Cited

```json
"total_mutants": 444,
"missed": 0,
"caught": 370,
"timeout": 0,
"unviable": 74
```
