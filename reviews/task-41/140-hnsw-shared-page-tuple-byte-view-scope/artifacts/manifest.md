# Artifact Manifest: Task 41 / Packet 140

- Head SHA: `b42091aec8942f50c3e7c81ad2f94daf8886fce9`
- Task bucket: `reviews/task-41/`
- Packet path: `reviews/task-41/140-hnsw-shared-page-tuple-byte-view-scope/`
- Lane: Task 41 invariant #2, Phase D, HNSW shared page tuple byte-view scope
- Fixture: local compile/static validation only
- Storage format: not applicable
- Rerank mode: not applicable
- Isolated one-index-per-table or shared-table surface: not applicable
- Timestamp: 2026-05-17 20:50 PDT

## Artifacts

### `cargo-fmt-check.log`

- Command: `script -q -e -c "cargo fmt --all --check" reviews/task-41/140-hnsw-shared-page-tuple-byte-view-scope/artifacts/cargo-fmt-check.log`
- Result: exit 0.
- Key lines: `Script done on 2026-05-17 20:50:41-07:00 [COMMAND_EXIT_CODE="0"]`.

### `cargo-check-pg18.log`

- Command: `script -q -e -c "cargo check --no-default-features --features pg18" reviews/task-41/140-hnsw-shared-page-tuple-byte-view-scope/artifacts/cargo-check-pg18.log`
- Result: exit 0.
- Key lines: `warning: ecaz (lib) generated 1 warning`; `Finished dev profile`; `Script done on 2026-05-17 20:50:41-07:00 [COMMAND_EXIT_CODE="0"]`.
- Note: the warning is the pre-existing unused import warning in `src/am/mod.rs`.

### `git-diff-check-head.log`

- Command: `script -q -e -c "git diff --check HEAD" reviews/task-41/140-hnsw-shared-page-tuple-byte-view-scope/artifacts/git-diff-check-head.log`
- Result: exit 0.
- Key lines: no diff-check output; `Script done on 2026-05-17 20:50:41-07:00 [COMMAND_EXIT_CODE="0"]`.
