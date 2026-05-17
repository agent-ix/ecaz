# 30776 - SPIRE CLI multicluster transport entrypoint

Review commit: `da8957031749bb735eb8c5a72822f735fa43fe28`

## Summary

This slice adds the first `ecaz` operator entrypoint for the local SPIRE
multi-instance fixture:

```text
ecaz dev spire-multicluster transport-overlap-pg18
```

The command wraps the existing reviewed
`scripts/run_spire_multicluster_transport_overlap_pg18.sh` harness, discovers
the local PG18 pgrx install when `--pgbin` is not supplied, and forwards the
packet-local artifact/log arguments reviewers need:

- `--artifact-dir`;
- `--smoke-log`;
- `--run-dir`;
- `--log-dir`;
- `--run-id`;
- explicit coordinator / fast remote / slow remote port overrides;
- `--skip-install` for reusing an already installed pg_test build.

This does not claim the full Stage E epoch/lifecycle/fault matrix is complete.
It moves the existing one-coordinator/two-remote transport-overlap setup and
teardown path behind the CLI boundary required by the production-readiness
workflow.

## Validation

All commands were run on `da8957031749bb735eb8c5a72822f735fa43fe28`.

```text
cargo fmt --check
cargo check -p ecaz-cli
cargo test -p ecaz-cli spire_multicluster -- --nocapture
git diff --check -- crates/ecaz-cli/src/commands/dev/mod.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs crates/ecaz-cli/README.md plan/tasks/task30-phase11-spire-distributed-production-parity.md
cargo run -p ecaz-cli -- dev spire-multicluster transport-overlap-pg18 --help
```

The focused CLI unit test verifies the new command parses with packet artifact
arguments and `--skip-install`. The help command verifies clap exposes the
operator surface without running the fixture.

## Review Questions

- Is wrapping the existing fixture script acceptable as the first `ecaz`
  operator entrypoint, given setup/teardown behavior stays in the reviewed
  harness?
- Are the forwarded artifact/log/run/port options sufficient for packet-local
  Stage E evidence capture?
- Should the next Stage E slice extend this command with fault/lifecycle
  subcommands, or should those get separate `ecaz dev spire-multicluster`
  subcommands?
