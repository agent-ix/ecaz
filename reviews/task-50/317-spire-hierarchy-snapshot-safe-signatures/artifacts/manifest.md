# Task 50 Packet 317 Artifact Manifest

- Head SHA: `c07a7dd5fae5e0ff80ad371956807333805728d2`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/317-spire-hierarchy-snapshot-safe-signatures`
- Timestamp: `2026-05-21T21:07:21Z`
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

### `no-unsafe-hierarchy-signatures.log`

- Command: `rg -n "pub\\(crate\\) unsafe fn index_(top_graph|hierarchy|object|delta|scan_placement|selected_pid_placement|scan_routing|root_routing|routing_centroid)_snapshot|pub\\(crate\\) unsafe fn classify_centroid|checked_live_index_relation" src/am/ec_spire/coordinator/hierarchy_snapshots.rs src/am/ec_spire/coordinator/snapshots.rs src/lib.rs src/am/ec_spire/cost/mod.rs`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1993`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/317-spire-hierarchy-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/317-spire-hierarchy-snapshot-safe-signatures unsafe-ledger`
- Key result: `wrote 1382 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/317-spire-hierarchy-snapshot-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1382 current unsafe rows`
