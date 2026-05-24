# Task 50 Packet 115 Artifact Manifest

- head SHA: `9cffbea32d5964fb2ea058bb8c8f338f0a9100e4`
- task bucket: `reviews/task-50`
- packet path: `reviews/task-50/115-ivf-pq-model-loader-boundary`
- timestamp: `2026-05-20T17:28:18-07:00`
- lane: IVF PQ/FastScan quantizer payload unsafe burndown
- fixture: static compile and unsafe-ledger validation
- storage format: source-only slice
- rerank mode: not applicable
- index surface: existing IVF insert and scan PQ/FastScan model-loading paths; no benchmark index created

## Artifacts

### `code-diff.patch`

- command: `git show --format=fuller --stat --patch 9cffbea3`
- result: captures the code commit that makes `load_pq_fastscan_model` safe and removes unsafe caller blocks in IVF insert/scan paths.

### `git-diff-check.log`

- command: `git diff --check HEAD~1..HEAD`
- result: passed; no whitespace errors.

### `src-unsafe-block-count-after.log`

- command: `make unsafe-block-count`
- result: aggregate direct unsafe count after this slice is `1665` across `124` files.

### `count-summary.md`

- command: `awk '{s += $1; f += 1} END {print "unsafe_blocks " s; print "files " f}' reviews/task-50/115-ivf-pq-model-loader-boundary/artifacts/src-unsafe-block-count-after.log`
- result:
  - `unsafe_blocks 1665`
  - `files 124`

### `cargo-check-pg18-bench.log`

- command: `cargo check --all-targets --no-default-features --features pg18,bench`
- result: passed. The log retains the known existing `src/am/mod.rs` SPIRE DML unused-import warning.

### `unsafe-ledger-after.jsonl`

- command: `make UNSAFE_LEDGER=reviews/task-50/115-ivf-pq-model-loader-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/115-ivf-pq-model-loader-boundary unsafe-ledger`
- result: generated packet-local ledger snapshot with `1665` rows.

### `unsafe-ledger-generate.log`

- command: `make UNSAFE_LEDGER=reviews/task-50/115-ivf-pq-model-loader-boundary/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/115-ivf-pq-model-loader-boundary unsafe-ledger`
- result: ledger generation completed successfully.

### `unsafe-ledger-check.log`

- command: `make UNSAFE_LEDGER=reviews/task-50/115-ivf-pq-model-loader-boundary/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check`
- result: passed with `ledger covers 1665 current unsafe rows`.
