head_sha: eebef54b864e42baa9b7c7a1773b125d6c14a3f8
task_bucket: reviews/task-85
packet_path: reviews/task-85/016-structured-snapshot-fallback
timestamp: 2026-06-07
lane: local-validation
fixture: none
storage_format: not applicable
rerank_mode: not applicable
surface: CLI compatibility for structured Postgres missing-column errors
isolated_one_index_per_table: not applicable

# Artifacts

- `cargo-fmt-check.log`
  - command: `cargo fmt --check`
  - result: passed with existing stable-rustfmt warnings.
- `cargo-test-ecaz-cli-spire-pipeline.log`
  - command: `cargo test --manifest-path crates/ecaz-cli/Cargo.toml spire_pipeline --locked --offline`
  - result: passed; 21 tests passed, 0 failed.

# Key Result Lines

- `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 385 filtered out`.

# Implementation Notes

Packet 015 matched only `tokio_postgres::Error::to_string()`. AWS rerun
evidence in packet 014 showed the actionable missing-column text was stored in
the structured `DbError` message. This checkpoint updates the guard to inspect
`err.as_db_error().message()` first, then fall back to display text.
