# Task 50 Packet 282 Artifact Manifest

- head SHA: `fd60e9bc837733f0fda829bc1a01baf47afd39c5`
- task bucket: `reviews/task-50/282-hnsw-gettuple-heap-tid-scoped-helper`
- timestamp: `2026-05-21T11:06:11-07:00`
- lane: Task 50 unsafe burndown, HNSW debug scan helpers
- fixture/storage/rerank mode: not applicable; code-level debug helper refactor
- surface isolation: not applicable; no benchmark matrix or table/index fixture used

## Artifacts

- `git-diff-check.log`
  - command: `git diff --check`
  - result: pass, exit 0
- `rustfmt-check.log`
  - command: `cargo fmt --all -- --check`
  - result: exit 1 due repo-wide pre-existing rustfmt drift outside this slice; retained as evidence rather than used as a clean gate
  - key lines: reports diffs in CLI bench code, hardening careful helpers, `src/quant/simd.rs`, `src/storage/relation_guard.rs`, and two existing HNSW formatting preferences
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
  - result: `2148`
- `src-unsafe-rg.log`
  - command: `rg -n unsafe src`
  - result: raw post-slice `src` unsafe listing for continued burndown
- `hnsw-remaining-direct-scan-debug-raw-reads.log`
  - command: `rg -n 'debug_scan_heap_tid\\(scan\\)|item_pointer_get_both\\(unsafe|debug_scan_opaque_mut\\(scan\\)' src/am/ec_hnsw/scan_debug.rs`
  - result: only the scoped wrapper internals and heap-TID witness helper remain for this raw-read pattern
- `git-diff-stat.log`
  - command: `git diff --stat`
  - result: one file touched before commit, `src/am/ec_hnsw/scan_debug.rs`

