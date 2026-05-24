# Task 50 Packet 283 Artifact Manifest

- head SHA: `6724b8bddfe6c154e9a961b7fd8708d9c77e6d72`
- task bucket: `reviews/task-50/283-hnsw-debug-palloc-orderby-score-guards`
- timestamp: `2026-05-21T11:10:59-07:00`
- lane: Task 50 unsafe burndown, HNSW debug scan helpers
- fixture/storage/rerank mode: not applicable; code-level debug helper refactor
- surface isolation: not applicable; no benchmark matrix or table/index fixture used

## Artifacts

- `git-diff-check.log`
  - command: `git diff --check`
  - result: pass, exit 0
- `cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: pass, exit 0
  - key lines: `Finished dev profile`; existing SPIRE DML re-export unused-import warning remains
- `cargo-test-lib-pg18-pgtest-no-run.log`
  - command: `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
  - result: pass, exit 0
  - key lines: `Finished test profile`; existing Hadamard test-only dead-code warnings remain
- `src-unsafe-count.log`
  - command: `rg -n unsafe src | wc -l`
  - result: `2145`
- `hnsw-palloc-orderby-score-raw-patterns.log`
  - command: `rg -n 'palloc0|pfree|xs_orderbyvals|xs_orderbynulls|debug_gettuple_orderby_score_slot|DebugPallocScanKey' src/am/ec_hnsw/scan_debug.rs`
  - result: confirms caller-facing palloc/free and one-off order-by raw reads are centralized
- `git-diff-stat.log`
  - command: `git diff --stat`
  - result: one file touched before commit, `src/am/ec_hnsw/scan_debug.rs`

