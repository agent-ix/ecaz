# Task 50 Packet 325 Artifact Manifest

- Head SHA: `683c25b7bb21ece095fcd89b8a7bd936fd7205f6`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/325-spire-custom-scan-cost-helper-safe-signature`
- Timestamp: `2026-05-21T21:42:58Z`
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

### `no-unsafe-custom-scan-cost-helper-signature.log`

- Command: `rg -n "unsafe fn estimate_custom_scan_cost|unsafe \\{[[:space:]]*estimate_custom_scan_cost\\(" src/am/ec_spire/custom_scan/cost_helpers.rs`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1944`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/325-spire-custom-scan-cost-helper-safe-signature/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/325-spire-custom-scan-cost-helper-safe-signature unsafe-ledger`
- Key result: `wrote 1360 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/325-spire-custom-scan-cost-helper-safe-signature/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1360 current unsafe rows`
