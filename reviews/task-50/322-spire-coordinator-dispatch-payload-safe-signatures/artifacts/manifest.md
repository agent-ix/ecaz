# Task 50 Packet 322 Artifact Manifest

- Head SHA: `eb530bd890626c8a587df23c3b1d3b7b4742d577`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/322-spire-coordinator-dispatch-payload-safe-signatures`
- Timestamp: `2026-05-21T21:32:19Z`
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

### `no-unsafe-coordinator-dispatch-payload-signatures.log`

- Command: `rg -n "pub\\(crate\\) unsafe fn coordinator_insert_dispatch_plan_row|coordinator_insert_dispatch_plan_row\\(index_relation|with_live_index_relation_safe|let dispatch = unsafe \\{ coordinator_insert_dispatch_plan_row|with_live_index_relation!\\([^\\n]*am::spire_coordinator_insert_dispatch_plan_row" src/am/ec_spire/coordinator/remote_candidates src/lib.rs src/tests`
- Key result: no matches; `rg` exit status `1`.

### `unsafe-line-count.log`

- Command: `rg -n "unsafe" src | wc -l`
- Key result: `1953`

### `unsafe-count-by-file.log`

- Command: `rg -n unsafe src --count-matches`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `make UNSAFE_LEDGER=reviews/task-50/322-spire-coordinator-dispatch-payload-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/322-spire-coordinator-dispatch-payload-safe-signatures unsafe-ledger`
- Key result: `wrote 1364 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `make UNSAFE_LEDGER=reviews/task-50/322-spire-coordinator-dispatch-payload-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- Key result: `ledger covers 1364 current unsafe rows`
