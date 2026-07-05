# Task 50 Packet 114 Artifact Manifest

- head SHA: `48dee69cd2e3a320f4d5c7bd148a74a5bb808735`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/114-ivf-admin-wrapper-boundaries`
- timestamp: `2026-05-20T17:22:47-07:00`
- lane: IVF admin / SQL diagnostic wrapper unsafe burndown
- fixture: static compile and unsafe-ledger validation
- storage format: source-only slice
- rerank mode: not applicable
- index surface: existing SQL diagnostic wrappers; no benchmark index created

## Artifacts

### `code-diff.patch`

- command: `git show --format=fuller --stat --patch 48dee69c`
- result: captures the code commit that converts IVF admin drift/admin/page-ownership facades to safe functions and switches three SQL wrappers to `with_live_index_relation_safe!`.

### `git-diff-check.log`

- command: `git diff --check HEAD~1..HEAD`
- result: passed; no whitespace errors.

### `src-unsafe-block-count-after.log`

- command: `make unsafe-block-count`
- result: aggregate direct unsafe count after this slice is `1667` across `124` files.

### `count-summary.md`

- command: `awk '{s += $1; f += 1} END {print "unsafe_blocks " s; print "files " f}' reviews/task-50/114-ivf-admin-wrapper-boundaries/artifacts/src-unsafe-block-count-after.log`
- result:
  - `unsafe_blocks 1667`
  - `files 124`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed. The log retains the known existing `src/am/mod.rs` SPIRE DML unused-import warning.

### `unsafe-ledger-after.jsonl`

- command: `make UNSAFE_LEDGER=reviews/task-50/114-ivf-admin-wrapper-boundaries/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/114-ivf-admin-wrapper-boundaries unsafe-ledger`
- result: generated packet-local ledger snapshot with `1667` rows.

### `unsafe-ledger-generate.log`

- command: `make UNSAFE_LEDGER=reviews/task-50/114-ivf-admin-wrapper-boundaries/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/114-ivf-admin-wrapper-boundaries unsafe-ledger`
- result: ledger generation completed successfully.

### `unsafe-ledger-check.log`

- command: `make UNSAFE_LEDGER=reviews/task-50/114-ivf-admin-wrapper-boundaries/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- result: passed with `ledger covers 1667 current unsafe rows`.
