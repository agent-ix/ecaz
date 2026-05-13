# Artifact Manifest: SPIRE Multicluster Smoke/Insert CLI Paths

- Head SHA: `95054735d2ac4b59022b0c8b03ea2171b4dc66cd`
- Packet/topic: `30968-spire-multicluster-cli-smoke-insert`
- Timestamp: `2026-05-13T04:57:37Z`
- Lane / fixture / storage format / rerank mode: non-live CLI wrapper/parser
  coverage for PG18 multicluster smoke and insert/read fixtures; storage format
  and rerank mode not exercised.
- Surface isolation: not applicable; no one-index-per-table or shared-table
  runtime fixture was started.

## Artifacts

### `git-diff-check.log`

- Command:
  `script -q -c "git diff --check 95054735^ 95054735" review/30968-spire-multicluster-cli-smoke-insert/artifacts/git-diff-check.log`
- Result lines:
  - Command exited successfully with no diff-check findings.

### `cargo-test-ecaz-cli-spire-multicluster.log`

- Command:
  `script -q -c "cargo test -p ecaz-cli spire_multicluster" review/30968-spire-multicluster-cli-smoke-insert/artifacts/cargo-test-ecaz-cli-spire-multicluster.log`
- Result lines:
  - `running 21 tests`
  - `test cli::tests::cli_parses_spire_multicluster_smoke_command ... ok`
  - `test cli::tests::cli_parses_spire_multicluster_insert_read_after_customscan_command ... ok`
  - `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 301 filtered out; finished in 0.01s`

### `cargo-check-pg18.log`

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30968-spire-multicluster-cli-smoke-insert/artifacts/cargo-check-pg18.log`
- Result lines:
  - `Finished 'dev' profile [unoptimized + debuginfo] target(s)`
  - Existing warning: `ecaz` lib has unused imports in `src/am/mod.rs`.
