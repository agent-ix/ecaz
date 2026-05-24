# Task 50 Packet 286 Artifact Manifest

- head SHA: `12d38ce1872e833fd58191ba97912194c5180a44`
- task bucket: `reviews/task-50/286-hnsw-oracle-scan-guard-orderby-state`
- timestamp: `2026-05-21T11:23:37-07:00`
- lane: Task 50 unsafe burndown, HNSW debug oracle scan helpers
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
  - result: `2137`
- `hnsw-oracle-orderby-patterns.log`
  - command: `rg -n 'DebugAmScan|with_oracle_score_parts|debug_scan_orderby_score_state|debug_scan_orderby_score\\(|debug_gettuple_orderby_score_slot|debug_with_oracle_score_parts\\\\(scan' src/am/ec_hnsw/scan_debug.rs`
  - result: shows oracle probes using `DebugAmScan`, and order-by score raw reads centralized through `debug_scan_orderby_score_state`
- `git-diff-stat.log`
  - command: `git diff --stat`
  - result: one file touched before commit, `src/am/ec_hnsw/scan_debug.rs`

