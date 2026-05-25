# Review Request: SPIRE Phase 13e Remote Identity Feedback Fix

Task: Task 30 Phase 13e
Code commit: `9c216d77544786c13d526bd129797b1b0676eb6b`

## Summary

This checkpoint addresses the blocking feedback from packets 961, 962, and
963:

- `distributed_remote_identity_query_sql` no longer emits `active_epoch`, so
  the generated remote identity JSON matches the renderer's strict
  `RemoteEndpointIdentity` contract.
- The renderer test coverage now includes the accepted identity shape and an
  explicit rejection of unexpected `active_epoch` input, keeping the generator
  and consumer contract fail-closed.
- `scripts/spire-aws/register.sh` no longer writes combined stdout/stderr into
  `node-*-identity.json`. Identity JSON is captured from stdout only, while
  stderr is stored separately as `node-*-identity.stderr.log`; `PGOPTIONS`
  lowers remote message noise during this machine-readable query.

## Validation

Artifacts are under
`reviews/task-30/965-spire-phase13e-remote-identity-feedback-fix/artifacts/`.

- `cargo test -p ecaz-cli commands::corpus::render_spire_registrations`
  passed: 7 passed, 0 failed.
- `cargo test -p ecaz-cli commands::corpus::load::tests::distributed_descriptor_registration_sql_uses_remote_endpoint_identity`
  passed: 1 passed, 0 failed.
- `bash -n scripts/spire-aws/register.sh` passed.
- `cargo fmt --all -- --check` passed with existing stable-rustfmt warnings for
  ignored nightly-only import options.
- `git diff --check HEAD` passed.

## Scope Notes

This fixes the registration feedback blocker only. Leaf-owned remote
materialization and parallel fanout are still outstanding Phase 13e work.
