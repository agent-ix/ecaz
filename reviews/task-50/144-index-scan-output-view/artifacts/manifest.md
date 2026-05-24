# Task 50 Packet 144 Artifact Manifest

- head SHA: `7eb1ec04535333cf0816b5f877023b20eb8842f5`
- task bucket: `reviews/task-50`
- packet: `reviews/task-50/144-index-scan-output-view`
- timestamp: `2026-05-20T19:46:28-07:00`
- lane: unsafe burndown soundness follow-up
- scope: AM scan output helpers for HNSW, IVF, and SPIRE
- isolated one-index-per-table/shared-table: not applicable; no benchmark run

## Artifacts

- `artifacts/code-stat.log`
  - command: `git show --stat --oneline --no-renames HEAD`
  - result: code commit `7eb1ec04` replaces safe raw `IndexScanDesc` output helpers with a lifetime-bound output view.
- `artifacts/code-diff.patch`
  - command: `git show --patch --no-ext-diff HEAD`
  - result: durable patch for the code checkpoint.
- `artifacts/git-diff-check.log`
  - command: `git diff --check HEAD^ HEAD`
  - result: passed with no whitespace diagnostics.
- `artifacts/cargo-check-pg18-bench.log`
  - command: `cargo check --all-targets --no-default-features --features pg18,bench`
  - result: passed; existing `src/am/mod.rs` unused SPIRE DML import warning remains.
- `artifacts/src-unsafe-block-count-after.log`
  - command: `make unsafe-block-count`
  - result: current direct unsafe count input for this packet.
- `artifacts/count-summary.md`
  - command: `awk '{s+=$1; f+=1} END {printf ...}' artifacts/src-unsafe-block-count-after.log`
  - result: `1550` direct unsafe blocks/functions across `124` files; packet delta `0`.
- `artifacts/unsafe-ledger-after.jsonl`
  - command: `make unsafe-ledger UNSAFE_LEDGER=reviews/task-50/144-index-scan-output-view/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/144-index-scan-output-view`
  - result: generated ledger for all current unsafe rows.
- `artifacts/unsafe-ledger-generate.log`
  - command: same ledger generation command above
  - result: command log for ledger generation.
- `artifacts/unsafe-ledger-check.log`
  - command: `make unsafe-ledger-check UNSAFE_LEDGER=reviews/task-50/144-index-scan-output-view/artifacts/unsafe-ledger-after.jsonl`
  - result: `ledger covers 1550 current unsafe rows`.
