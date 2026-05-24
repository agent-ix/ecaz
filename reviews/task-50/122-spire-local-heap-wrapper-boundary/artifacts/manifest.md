# Task 50 Packet 122 Artifact Manifest

- head SHA: `bdd090a526f1a682be949c36f87ae7f95dec912a`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/122-spire-local-heap-wrapper-boundary`
- timestamp: `2026-05-20T17:53:19-07:00`
- lane: SPIRE local heap wrapper unsafe burndown
- fixture: static compile and unsafe-ledger validation
- storage format: source-only slice
- rerank mode: not applicable
- index surface: SPIRE remote-search local heap plan/candidate SQL wrappers and pipeline summary; no benchmark index created

## Artifacts

### `code-diff.patch`

- command: `git show --format=fuller --stat --patch bdd090a5`
- result: captures the code commit that makes local heap wrapper APIs safe while retaining the actual heap-fetch unsafe boundary internally.

### `git-diff-check.log`

- command: `git diff --check HEAD~1..HEAD`
- result: passed; no whitespace errors.

### `src-unsafe-block-count-after.log`

- command: `make unsafe-block-count`
- result: aggregate direct unsafe count after this slice is `1638` across `123` files.

### `count-summary.md`

- command: `awk '{s += $1; f += 1} END {print "unsafe_blocks " s; print "files " f}' reviews/task-50/122-spire-local-heap-wrapper-boundary/artifacts/src-unsafe-block-count-after.log`
- result:
  - `unsafe_blocks 1638`
  - `files 123`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed. The log retains the known existing `src/am/mod.rs` SPIRE DML unused-import warning.

### `unsafe-ledger-after.jsonl`

- command: `make UNSAFE_LEDGER=reviews/task-50/122-spire-local-heap-wrapper-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/122-spire-local-heap-wrapper-boundary unsafe-ledger`
- result: generated packet-local ledger snapshot with `1638` rows.

### `unsafe-ledger-generate.log`

- command: `make UNSAFE_LEDGER=reviews/task-50/122-spire-local-heap-wrapper-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/122-spire-local-heap-wrapper-boundary unsafe-ledger`
- result: ledger generation completed successfully.

### `unsafe-ledger-check.log`

- command: `make UNSAFE_LEDGER=reviews/task-50/122-spire-local-heap-wrapper-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- result: passed with `ledger covers 1638 current unsafe rows`.
