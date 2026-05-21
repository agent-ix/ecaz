# Task 50 Packet 315 Artifact Manifest

- Head SHA: `4dd646646409643bdb7b98b12d32834b6f728a2c`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/315-spire-remote-search-wrapper-safe-signatures`
- Timestamp: `2026-05-21T20:31:35Z`
- Lane: SPIRE unsafe burndown
- Fixture / storage format / rerank mode: not applicable
- Surface: code review packet; no benchmark matrix
- Isolation: not applicable; no table/index benchmark surface

## Artifacts

### `git-diff-check.log`

- Command: `script -q -c "git diff --check HEAD~2..HEAD" reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/git-diff-check.log`
- Result: passed

### `cargo-check-pg18-bench.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/cargo-check-pg18-bench.log`
- Result: passed
- Note: emitted the pre-existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`.

### `unsafe-line-count.log`

- Command: `script -q -c "rg -n unsafe src | wc -l" reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/unsafe-line-count.log`
- Key result: `2001`

### `unsafe-count-by-file.log`

- Command: `script -q -c "rg -n unsafe src --count-matches" reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/unsafe-count-by-file.log`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `script -q -c "make UNSAFE_LEDGER=reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/315-spire-remote-search-wrapper-safe-signatures unsafe-ledger" reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/unsafe-ledger-generate.log`
- Key result: `wrote 1367 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `script -q -c "make UNSAFE_LEDGER=reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/315-spire-remote-search-wrapper-safe-signatures/artifacts/unsafe-ledger-check.log`
- Key result: `ledger covers 1367 current unsafe rows`
