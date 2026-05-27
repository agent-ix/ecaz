# Artifact Manifest

- Head SHA: `e95913f0dda2ddf6a6465ee10c9ef9cb191b53c8`
- Task bucket: `reviews/task-30/1022-spire-phase13e-suite-pgoptions-evidence`
- Timestamp: `2026-05-27T10:41:45-07:00`
- Lane: Phase 13e representative pooling evidence visibility
- Fixture / storage / rerank mode: local suite dry-run only
- Isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `git-show-stat.log`

- Command: `git show --stat --oneline --decorate --no-renames e95913f0d`
- Key lines:
  - `e95913f0d (HEAD -> diskann-aws-optimization) Show suite PGOPTIONS in run output`
  - `crates/ecaz-cli/src/commands/bench/suite.rs | 39 ++++++++++++++++++++++++++---`
  - `1 file changed, 36 insertions(+), 3 deletions(-)`

### `cargo-fmt-check.log`

- Command: `cargo fmt --check`
- Result: passed.
- Note: the log includes existing stable-toolchain warnings for nightly-only
  rustfmt options.

### `cargo-test-pgoptions.log`

- Command: `cargo test -p ecaz-cli shell_join_with_pgoptions_renders_environment_prefix`
- Key line:
  - `test commands::bench::suite::tests::shell_join_with_pgoptions_renders_environment_prefix ... ok`

### `pooling-suite-dry-run.log`

- Command: `target/debug/ecaz bench suite --config scripts/spire-aws/suite-representative-pooling.json --dry-run`
- Key lines:
  - `13e4-pooling-disabled-profile-k10 -> PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.remote_search_connection_pool_size=0" --database tqvector_bench bench spire-pipeline ...`
  - `13e4-pooling-enabled-profile-k10 -> PGOPTIONS="-c enable_seqscan=off -c enable_indexscan=off -c ec_spire.remote_search_connection_pool_size=16" --database tqvector_bench bench spire-pipeline ...`

## Notes

The validation dry-run generated
`scripts/spire-aws/artifacts/representative-pooling/suite-manifest.json`.
That file was left in place and was not staged.
