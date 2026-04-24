# Review Request: PG18 Parallel Preflight Local GUCs

Current head: `663ad81aa14d021ed9e80e7511add522ebeefdaa`

Scope:
- `crates/ecaz-cli/src/commands/dev/test.rs`
- `crates/ecaz-cli/src/commands/dev/install.rs`
- `crates/ecaz-cli/src/commands/dev/scratch.rs`
- `crates/ecaz-cli/src/commands/dev/support.rs`

Problem:
- The PG18 parallel-scan preflight's seqscan control plan mutated planner GUCs
  on the shared CLI session with plain `SET`.
- Today that control query is the last ordered-path use in the command, so it
  was not a correctness bug, but future additions could accidentally inherit the
  control plan's `enable_*` state.
- Running clippy on the ecaz-cli package also exposed small existing hygiene
  issues in the dev support code.

What changed:
- Runs the seqscan control plan inside a transaction and uses `SET LOCAL` for
  the control-only planner GUCs, so the candidate ordered-path session state is
  restored automatically after the control query.
- Keeps the same diagnostic output and `--expect-parallel` behavior.
- Cleans up ecaz-cli clippy findings by accepting `&Path` instead of
  `&PathBuf` in dev helpers and removing a needless `Ok(?)` wrapper.

Artifact:
- `artifacts/pg18-parallel-scan.log` captures the PG18 CLI preflight after the
  GUC-localization change.
- The log still shows ordered candidate IDs matching serial IDs and a parallel
  seqscan control plan with 4 workers launched.

Validation:
- Passed:
  - `cargo fmt`
  - `git diff --check`
  - `cargo check -p ecaz-cli`
  - `cargo test -p ecaz-cli`
  - `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
  - `cargo run -p ecaz-cli -- dev test pg18-parallel-scan`

Review focus:
- Whether using a transaction with `SET LOCAL` is the right boundary for the
  control-plan-only GUC overrides.
- Whether the incidental clippy cleanups are appropriately scoped for this CLI
  hardening checkpoint.
