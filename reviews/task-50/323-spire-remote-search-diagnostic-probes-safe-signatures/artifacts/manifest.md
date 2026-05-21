# Task 50 Packet 323 Artifact Manifest

- Head SHA: `d3f782ea0805cca12e77eaae405fef961d78c032`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/323-spire-remote-search-diagnostic-probes-safe-signatures`
- Timestamp: `2026-05-21T21:35:44Z`
- Lane: SPIRE unsafe burndown
- Fixture / storage format / rerank mode: not applicable
- Surface: code review packet; no benchmark matrix
- Isolation: not applicable; no table/index benchmark surface

## Artifacts

### `git-diff-check.log`

- Command: `git diff --check HEAD~1..HEAD`
- Result: passed

### `cargo-check-pg18-bench.log`

- Command: `cargo check --all-targets --no-default-features --features pg18,bench`
- Result: passed
- Note: emitted the pre-existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`.

### `no-unsafe-remote-search-diagnostic-probe-signatures.log`

- Command: `rg -n "unsafe fn remote_search_libpq_identity_cache_contract_probe_counts|unsafe fn remote_search_operator_diagnostics_row|with_live_index_relation!\\([^\\n]*am::spire_remote_search_operator_diagnostics_row|unsafe \\{[[:space:]]*am::spire_remote_search_libpq_identity_cache_contract_probe_counts|let index = unsafe \\{ live_index_relation\\(index_relation\\) \\}" src/am/ec_spire/coordinator/remote_candidates src/lib.rs src/tests`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1949`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/323-spire-remote-search-diagnostic-probes-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/323-spire-remote-search-diagnostic-probes-safe-signatures unsafe-ledger`
- Key result: `wrote 1362 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/323-spire-remote-search-diagnostic-probes-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1362 current unsafe rows`
