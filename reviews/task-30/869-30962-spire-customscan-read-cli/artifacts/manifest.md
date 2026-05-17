# Artifact Manifest: SPIRE CustomScan Read CLI Wrapper

- head SHA: `0dad10d356c02feeef1808c2994d9f0dfa84e925`
- packet/topic: `30962-spire-customscan-read-cli`
- lane / fixture / storage format / rerank mode: CLI parser coverage for
  `ecaz dev spire-multicluster customscan-read-pg18`; no PostgreSQL fixture,
  storage format, or rerank mode was exercised.
- isolated one-index-per-table or shared-table surfaces: not applicable; this
  packet validates CLI routing only.

## Artifacts

### `cargo-test-cli-customscan-read.log`

- command: `cargo test -p ecaz-cli cli_parses_spire_multicluster_customscan_read_command`
- timestamp: `2026-05-12 21:01:04-07:00`
- key result lines:
  - `test cli::tests::cli_parses_spire_multicluster_customscan_read_command ... ok`
  - `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 312 filtered out; finished in 0.00s`
- note: the run reported a pre-existing `ecaz` library unused-import warning in
  `src/am/mod.rs`; the CLI parser test passed.

### `git-diff-check.log`

- command: `git diff --check -- crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs crates/ecaz-cli/README.md plan/tasks/task30-phase12-spire-production-hardening.md`
- timestamp: `2026-05-12 21:02:08-07:00`
- key result lines:
  - command exited with status `0`
