# Task 121 Phase 0 Local Multinode Feedback Fix Artifacts

- head_sha: af885a363abd8eff9f99f670c18047a1039eff3e
- task_bucket: reviews/task-121
- packet: reviews/task-121/004-phase0-local-multinode-feedback-fix
- scope: local-only Phase 0 feedback fix for `spire-local-multinode` suite matrix controls and artifact tracking
- timestamp: 2026-06-23 05:09 America/Los_Angeles
- storage formats in dry-run: turboquant
- isolated surfaces: not applicable; dry-run only, no indexes built

## Artifacts

### cargo-test-ecaz-cli-suite.log

- command: `script -q -c "cargo test -p ecaz-cli commands::bench::suite" reviews/task-121/004-phase0-local-multinode-feedback-fix/artifacts/cargo-test-ecaz-cli-suite.log`
- result: PASS
- key lines:
  - `running 54 tests`
  - `test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 360 filtered out`
  - `COMMAND_EXIT_CODE="0"`

### cargo-build-ecaz-cli.log

- command: `script -q -c "cargo build -p ecaz-cli --bin ecaz" reviews/task-121/004-phase0-local-multinode-feedback-fix/artifacts/cargo-build-ecaz-cli.log`
- result: PASS
- key lines:
  - `warning: field path is never read` in `crates/ecaz-cli/src/commands/corpus/load.rs`
  - `Finished dev profile [unoptimized + debuginfo] target(s) in 42.52s`
  - `COMMAND_EXIT_CODE="0"`

### suite-phase0-local-multinode-matrix-dryrun.json

- command config for a local multi-node matrix cell.
- dry-run only; uses `storage_format=turboquant`, shared reloptions, coordinator-only reloptions, and remote-only reloptions.

### suite-phase0-local-multinode-matrix-dryrun.log

- command: `script -q -c "target/debug/ecaz bench suite run --dry-run --config reviews/task-121/004-phase0-local-multinode-feedback-fix/artifacts/suite-phase0-local-multinode-matrix-dryrun.json --database postgres --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-121/004-phase0-local-multinode-feedback-fix/artifacts/suite-phase0-local-multinode-matrix-dryrun-manifest.json" reviews/task-121/004-phase0-local-multinode-feedback-fix/artifacts/suite-phase0-local-multinode-matrix-dryrun.log`
- result: PASS
- key generated command includes:
  - `--storage-format turboquant`
  - `--coord-index task121_phase0_coord_idx`
  - `--remote-index task121_phase0_remote_idx`
  - `--reloption nlists=128`
  - `--reloption recursive_fanout=8`
  - `--reloption top_graph_enabled=1`
  - `--coord-reloption training_sample_rows=10000`
  - `--remote-reloption boundary_replica_count=1`
  - `--skip-bench-suite`

### suite-phase0-local-multinode-matrix-dryrun-manifest.json

- generated manifest for the dry-run suite config.
- config_sha256: `739945b25ef6e04b0c384b47d6093876502000e6241b2042e7b6da985ef7caa5`
- expected_artifacts with `skip_bench_suite=true`:
  - `reviews/task-121/004-phase0-local-multinode-feedback-fix/artifacts/local-multinode-matrix-cell/local-multinode.log`
  - `target/spire-phase13e-aws-local-task121-phase0-matrix/topology.local.json`
- no nested `bench-suite/*` artifacts are declared for this skipped bench run.
