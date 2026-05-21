# Task 50 Packet 316 Artifact Manifest

- Head SHA: `86783b6d0edeb7a715dacdcc18ee4513db4dfe45`
- Task bucket: `reviews/task-50`
- Packet path: `reviews/task-50/316-spire-live-relation-wrapper-soundness-repair`
- Timestamp: `2026-05-21T20:59:38Z`
- Lane: SPIRE unsafe burndown soundness repair
- Fixture / storage format / rerank mode: not applicable
- Surface: code review packet; no benchmark matrix
- Isolation: not applicable; no table/index benchmark surface

## Artifacts

### `git-diff-check.log`

- Command: `script -q -c "git diff --check HEAD~1..HEAD" reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/git-diff-check.log`
- Result: passed

### `cargo-check-pg18-bench.log`

- Command: `script -q -c "cargo check --all-targets --no-default-features --features pg18,bench" reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/cargo-check-pg18-bench.log`
- Result: passed
- Note: emitted the pre-existing SPIRE DML re-export unused-import warning in `src/am/mod.rs`.

### `unsafe-line-count.log`

- Command: `script -q -c "rg -n unsafe src | wc -l" reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/unsafe-line-count.log`
- Key result: `2003`

### `unsafe-count-by-file.log`

- Command: `script -q -c "rg -n unsafe src --count-matches" reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/unsafe-count-by-file.log`
- Key result: packet-local per-file unsafe match counts captured for review.

### `unsafe-ledger-after.jsonl`

- Command: `script -q -c "make UNSAFE_LEDGER=reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/unsafe-ledger-after.jsonl UNSAFE_LEDGER_PACKET=reviews/task-50/316-spire-live-relation-wrapper-soundness-repair unsafe-ledger" reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/unsafe-ledger-generate.log`
- Key result: `wrote 1382 unsafe ledger rows`

### `unsafe-ledger-check.log`

- Command: `script -q -c "make UNSAFE_LEDGER=reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/unsafe-ledger-after.jsonl unsafe-ledger-check" reviews/task-50/316-spire-live-relation-wrapper-soundness-repair/artifacts/unsafe-ledger-check.log`
- Key result: `ledger covers 1382 current unsafe rows`
