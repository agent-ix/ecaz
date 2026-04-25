# Review Request: Generic ecaz dev SQL Runner

Current head: `d5e404b`

Scope:
- `crates/ecaz-cli/src/commands/dev/sql.rs`
- `crates/ecaz-cli/src/commands/dev/mod.rs`
- `crates/ecaz-cli/src/commands/dev/scratch.rs`
- `crates/ecaz-cli/src/commands/dev/support.rs`
- `crates/ecaz-cli/src/commands/dev/test.rs`
- `crates/ecaz-cli/README.md`

Problem:
- Manual SQL runs for PG18 were falling back to `script` or direct `psql`
  invocation because the existing dev SQL surface was too scratch-specific.
- We need a CLI-owned way to run packet-local SQL against local pgrx installs,
  with packet-local log output, without shell redirection.
- The CLI should be PostgreSQL-version-aware rather than PG17-focused. PG18 is
  the current default target, while `--pg 17` remains available for compatibility
  lanes.

What changed:
- Added `ecaz dev sql`.
- Supports:
  - `--pg` for PG major version, defaulting to 18
  - `--database` global database selection
  - `--socket-dir`
  - `--port`, defaulting to pgrx convention `28800 + pg`
  - `--sql`
  - `--file`
  - `--raw`
  - repeatable `--env NAME=VALUE`
  - `--log-output PATH`
- `--log-output` captures combined psql stdout/stderr into the requested file
  and echoes output back to the terminal.
- Moved pgrx port selection into a shared helper and made `ecaz dev test pgrx`
  default to PG18.
- Made `ecaz dev scratch restart/sql` accept `--pg`, defaulting to PG18, while
  preserving explicit `--pg 17` support.
- Documented the intended packet-local SQL/log-output usage in the CLI README.

Validation:
- Passed:
  - `cargo test -p ecaz-cli dev::sql`
  - `cargo test -p ecaz-cli dev::`
  - `git diff --check`

Review focus:
- Whether `dev sql` is the right generic surface rather than extending the
  scratch command further.
- Whether `--log-output` behavior is suitable for review-packet artifacts.
- Whether PG18 should be the default for generic dev/test commands while older
  versions remain opt-in via `--pg`.
