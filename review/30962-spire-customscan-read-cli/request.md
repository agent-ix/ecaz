# Review Request: SPIRE CustomScan Read CLI Wrapper

## Summary

Closes the Phase 12.7 row:

> Add or extend an `ecaz`-owned local one-coordinator/two-remote setup and
> teardown command for repeated Stage E and readiness runs.

This slice adds `ecaz dev spire-multicluster customscan-read-pg18`, a CLI-owned
entry point for the existing PG18 CustomScan readiness fixture. The existing
`transport-overlap-pg18` command remains the two-remote setup/teardown fixture;
the new command closes the repeated CustomScan readiness path under the same
`ecaz dev spire-multicluster` surface.

## Files

- `crates/ecaz-cli/src/commands/dev/spire_multicluster.rs`
- `crates/ecaz-cli/src/cli.rs`
- `crates/ecaz-cli/README.md`
- `plan/tasks/task30-phase12-spire-production-hardening.md`
- `review/30962-spire-customscan-read-cli/artifacts/manifest.md`
- `review/30962-spire-customscan-read-cli/artifacts/cargo-test-cli-customscan-read.log`
- `review/30962-spire-customscan-read-cli/artifacts/git-diff-check.log`

## Validation

- `cargo fmt --package ecaz-cli`
- `cargo test -p ecaz-cli cli_parses_spire_multicluster_customscan_read_command`
- `git diff --check -- crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs crates/ecaz-cli/README.md plan/tasks/task30-phase12-spire-production-hardening.md`

The test is intentionally parser-level only: it pins the CLI surface without
starting PostgreSQL, which avoids the sandbox approval path for Unix-domain
socket binding.

## Reviewer Focus

- Confirm the new CLI command forwards the same operator knobs as the
  underlying CustomScan read script.
- Confirm the tracker closure is scoped to the local setup/teardown command
  surface, not to the remaining Phase 12.9 benchmark/readiness bundle rows.
