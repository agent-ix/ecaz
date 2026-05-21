# Task 50 Packet 288 Artifact Manifest

- head SHA: `ecf0427cfd875798331360a605856ee10911e54d`
- task bucket: `reviews/task-50/288-hnsw-debug-graph-collectors-safe`
- timestamp: `2026-05-21T11:31:22-07:00`
- lane: Task 50 unsafe burndown, HNSW debug graph collectors
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
  - result: `2124`
- `hnsw-graph-collector-patterns.log`
  - command: `rg -n 'debug_collect_element_tids_at_level|debug_collect_element_tids_at_or_above_level|debug_collect_element_tid_by_heap_tid|debug_with_page_line_tuple_bytes|unsafe \\\\{ debug_collect_element|unsafe fn debug_collect_element' src/am/ec_hnsw/scan_debug.rs`
  - result: confirms collector functions and their callers are no longer unsafe APIs/blocks
- `git-diff-stat.log`
  - command: `git diff --stat`
  - result: one file touched before commit, `src/am/ec_hnsw/scan_debug.rs`

