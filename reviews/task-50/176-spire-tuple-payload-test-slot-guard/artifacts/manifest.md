# Task 50 Packet 176 Artifact Manifest

- head SHA: `b37c3024487aef223665531fe6e9ec8979676c3f`
- task bucket: `reviews/task-50/176-spire-tuple-payload-test-slot-guard`
- lane: SPIRE CustomScan unsafe burndown
- fixture / storage format / rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable
- timestamp: 2026-05-20 23:14 America/Los_Angeles

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/cargo-check-pg18-bench.log`
  - result: passed
  - key lines: `Finished dev profile`; existing `src/am/mod.rs` unused-import warning remains.

- `cargo-check-pg18-pg-test.log`
  - command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,pg_test" reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/cargo-check-pg18-pg-test.log`
  - result: passed
  - key lines: `Finished dev profile`; existing `src/quant/hadamard.rs` test-helper dead-code warnings remain.

- `git-diff-check.log`
  - command: `script -q -e -c "git diff --check HEAD~1..HEAD" reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/git-diff-check.log`
  - result: passed

- `unsafe-block-count.log`
  - command: `script -q -e -c "make unsafe-block-count" reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/unsafe-block-count.log`
  - result: passed
  - key lines: `src/tests/custom_scan.rs` now `13`; `src/am/ec_spire/custom_scan/tuple_payload.rs` remains `6`.

- `unsafe-ledger-generate.log`
  - command: `script -q -e -c "make UNSAFE_LEDGER=reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/176-spire-tuple-payload-test-slot-guard unsafe-ledger" reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/unsafe-ledger-generate.log`
  - result: passed
  - key line: `wrote 1849 unsafe ledger rows`.

- `unsafe-ledger-check.log`
  - command: `script -q -e -c "make UNSAFE_LEDGER=reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/176-spire-tuple-payload-test-slot-guard/artifacts/unsafe-ledger-check.log`
  - result: passed
  - key line: `ledger covers 1849 current unsafe rows`.

- `unsafe-ledger-after.jsonl`
  - result: generated ledger snapshot after `b37c3024487aef223665531fe6e9ec8979676c3f`
  - key result: `1849` current unsafe rows.
