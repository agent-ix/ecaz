# Task 50 Packet 319 Artifact Manifest

- Head SHA: `28d8b1a6a87d77bd5869fba453e1a553f83a6de0`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/319-spire-maintenance-snapshot-safe-signatures`
- Timestamp: `2026-05-21T21:16:21Z`
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

### `no-unsafe-maintenance-signatures.log`

- Command: `rg -n "pub\\(crate\\) unsafe fn index_(locked_)?maintenance|with_live_index_relation!\\([^\\n]*am::spire_index_.*maintenance|checked_live_index_relation" src/am/ec_spire/coordinator/maintenance.rs src/lib.rs`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1973`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/319-spire-maintenance-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/319-spire-maintenance-snapshot-safe-signatures unsafe-ledger`
- Key result: `wrote 1375 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/319-spire-maintenance-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1375 current unsafe rows`
