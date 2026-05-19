# Task 40 Model-Checking Completion Artifact Manifest

- Head SHA: `a587790950c7f06d51768f1dea23421c62bbeb13`
- Task bucket: `reviews/task-40/`
- Packet path: `reviews/task-40/002-model-checking-completion/`
- Timestamp: `2026-05-18T18:12:30-07:00`
- Isolated one-index-per-table vs shared-table surface: not applicable. These
  are pgrx-free model-checking and compile-validation lanes, not live
  PostgreSQL index/table measurement runs.

## Artifacts

### `loom-real.log`

- Lane: `loom-real`
- Fixture/storage/rerank mode: path-lifted pure Rust Loom harness for
  parallel worker slots and HNSW concurrent DSM node-insert state.
- Command: `bash scripts/hardening.sh loom-real`
- Key result: `test result: ok. 6 passed; 0 failed`.

### `shuttle-real.log`

- Lane: `shuttle-real`
- Fixture/storage/rerank mode: path-lifted Shuttle harness for SPIRE remote
  candidate merge and epoch-publish visibility helpers.
- Command: `bash scripts/hardening.sh shuttle-real`
- Key result: `test result: ok. 2 passed; 0 failed`.

### `sim-spire-remote.log`

- Lane: `sim-spire-remote`
- Fixture/storage/rerank mode: Turmoil UDP simulation over the pgrx-free SPIRE
  remote transport simulation model.
- Command: `bash scripts/hardening.sh sim-spire-remote`
- Key result: `test result: ok. 5 passed; 0 failed`.

### `production-merge-no-run.log`

- Lane: production compile validation.
- Fixture/storage/rerank mode: `cargo test --lib` compile of the production
  lib test binary with the SPIRE compact merge filter selected.
- Command:
  `cargo test --lib production_executor_compact_merge_uses_ready_candidate_batches --no-run`
- Key result: `Finished test profile`; executable produced for
  `target/debug/deps/ecaz-05fcddf1cff0198d`.
- Notes: the only warnings are the pre-existing Hadamard test helper
  `dead_code` warnings.

### `hardening-validate.log`

- Lane: governance validation.
- Fixture/storage/rerank mode: hardening lane inventory and synthetic-lane
  guard.
- Command: `bash scripts/hardening_validate.sh`
- Key result: command exited with `COMMAND_EXIT_CODE="0"`.
