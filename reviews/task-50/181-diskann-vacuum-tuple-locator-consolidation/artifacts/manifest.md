# Task 50 Packet 181 Artifact Manifest

- head SHA: `ff6e6c2e962bf46f029620f96f62b11578356a1b`
- task bucket: `reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation`
- lane: DiskANN unsafe burndown
- fixture / storage format / rerank mode: not applicable
- isolated one-index-per-table or shared-table surface: not applicable
- timestamp: 2026-05-21 00:08 America/Los_Angeles

## Artifacts

- `cargo-check-pg18-bench.log`
  - command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/cargo-check-pg18-bench.log`
  - result: passed
  - key lines: `Finished dev profile`; existing `src/am/mod.rs` unused-import warning remains.

- `cargo-check-pg18-pg-test.log`
  - command: `script -q -e -c "cargo check --all-targets --no-default-features --features pg18,pg_test" reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/cargo-check-pg18-pg-test.log`
  - result: passed
  - key lines: `Finished dev profile`; existing `src/quant/hadamard.rs` test-helper dead-code warnings remain.

- `git-diff-check.log`
  - command: `script -q -e -c "git diff --check HEAD~1..HEAD" reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/git-diff-check.log`
  - result: passed

- `unsafe-block-count.log`
  - command: `script -q -e -c "make unsafe-block-count" reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/unsafe-block-count.log`
  - result: passed
  - key line: `src/am/ec_diskann/routine.rs` now `54`.

- `unsafe-ledger-generate.log`
  - command: `script -q -e -c "make UNSAFE_LEDGER=reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation unsafe-ledger" reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/unsafe-ledger-generate.log`
  - result: passed
  - key line: `wrote 1827 unsafe ledger rows`.

- `unsafe-ledger-check.log`
  - command: `script -q -e -c "make UNSAFE_LEDGER=reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/181-diskann-vacuum-tuple-locator-consolidation/artifacts/unsafe-ledger-check.log`
  - result: passed
  - key line: `ledger covers 1827 current unsafe rows`.

- `unsafe-ledger-after.jsonl`
  - result: generated ledger snapshot after `ff6e6c2e962bf46f029620f96f62b11578356a1b`
  - key result: `1827` current unsafe rows.
