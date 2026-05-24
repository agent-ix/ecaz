# Task 50 Packet 324 Artifact Manifest

- Head SHA: `c63be45fa480b4e01438514d9d1b64677687ebbe`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/324-spire-custom-scan-eligibility-safe-signatures`
- Timestamp: `2026-05-21T21:39:54Z`
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

### `no-unsafe-custom-scan-eligibility-signatures.log`

- Command: `rg -n "unsafe fn custom_scan_index_eligibility_result|unsafe fn load_custom_scan_placement_directory|unsafe \\{ custom_scan_index_eligibility_result\\(|with_live_index_relation!\\([^\\n]*am::spire_custom_scan_index_eligibility_result" src/am/ec_spire/custom_scan src/lib.rs`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1946`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/324-spire-custom-scan-eligibility-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/324-spire-custom-scan-eligibility-safe-signatures unsafe-ledger`
- Key result: `wrote 1361 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/324-spire-custom-scan-eligibility-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1361 current unsafe rows`
