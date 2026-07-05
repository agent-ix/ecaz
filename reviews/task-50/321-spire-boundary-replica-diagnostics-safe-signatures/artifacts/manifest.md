# Task 50 Packet 321 Artifact Manifest

- Head SHA: `b19efa9de17bc52a4bf55f9ec3381468ede941ff`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/321-spire-boundary-replica-diagnostics-safe-signatures`
- Timestamp: `2026-05-21T21:24:41Z`
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

### `no-unsafe-boundary-replica-signatures.log`

- Command: `rg -n "pub\\(crate\\) unsafe fn index_boundary_replica|with_live_index_relation!\\([^\\n]*am::spire_index_boundary_replica|checked_live_index_relation" src/am/ec_spire/coordinator/diagnostics.rs src/lib.rs`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1958`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/321-spire-boundary-replica-diagnostics-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/321-spire-boundary-replica-diagnostics-safe-signatures unsafe-ledger`
- Key result: `wrote 1368 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/321-spire-boundary-replica-diagnostics-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1368 current unsafe rows`
