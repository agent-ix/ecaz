# Task 50 Packet 285 Artifact Manifest

- head SHA: `4a09f35d767a6f4080e3407d23d28a4236ad2cf5`
- task bucket: `reviews/task-50/285-hnsw-debug-am-scan-rescan-guard-reuse`
- timestamp: `2026-05-21T11:18:22-07:00`
- lane: Task 50 unsafe burndown, HNSW debug scan lifecycle helpers
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
  - result: `2138`
- `hnsw-am-scan-rescan-guard-patterns.log`
  - command: `rg -n 'DebugAmScan|debug_scan_has_opaque|debug_scan_opaque_is_null|debug_rescan_query_dimensions|debug_rescan_overwrites_query_dimensions|debug_rescan_with_unused_key_buffer|debug_gettuple_after_rescan_result' src/am/ec_hnsw/scan_debug.rs`
  - result: confirms normal rescan/gettuple probes route through `DebugAmScan` and raw opaque helper functions are gone
- `git-diff-stat.log`
  - command: `git diff --stat`
  - result: one file touched before commit, `src/am/ec_hnsw/scan_debug.rs`

