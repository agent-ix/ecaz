# Review Request: SPIRE Suite PGOPTIONS Evidence

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `e95913f0dda2ddf6a6465ee10c9ef9cb191b53c8`

## Summary

This checkpoint makes `ecaz bench suite` dry-run, run, and status output show
step-level `PGOPTIONS` when a suite step configures it.

The Phase 13e representative pooling suite already used `pgoptions` to force
the A/B settings, but the operator-visible dry-run output only printed the
child command. That made the pooling evidence ambiguous in review logs. The
pooling dry-run now visibly distinguishes:

- disabled: `ec_spire.remote_search_connection_pool_size=0`
- enabled: `ec_spire.remote_search_connection_pool_size=16`

No AWS was started.

## Validation

- `cargo fmt --check`
- `cargo test -p ecaz-cli shell_join_with_pgoptions_renders_environment_prefix`
- `cargo build -p ecaz-cli --bin ecaz`
- `target/debug/ecaz bench suite --config scripts/spire-aws/suite-representative-pooling.json --dry-run`

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-show-stat.log`
- `artifacts/cargo-fmt-check.log`
- `artifacts/cargo-test-pgoptions.log`
- `artifacts/pooling-suite-dry-run.log`

The dry-run also generated
`scripts/spire-aws/artifacts/representative-pooling/suite-manifest.json`; it
was intentionally left in place and is not part of this review packet.
