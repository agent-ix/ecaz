# Review Request: SPIRE Phase 13e Descriptor Registration Plan

## Summary

This checkpoint adds descriptor-registration SQL to the distributed placement
plan. Each remote entry now includes the SQL needed to query the remote endpoint
identity and a coordinator registration template that uses the real remote
`profile_fingerprint` as `remote_index_identity`.

Code commit: `e02f0ce310b7dd3d8e205882e9f9f40e9b6b751d`

## Changes

- Added per-remote `remote_identity_query_sql` to distributed placement output.
- The query reads:
  - `ec_spire_remote_search_endpoint_identity(remote_index)`
  - `ec_spire_index_active_snapshot_diagnostics(remote_index)`
- Added per-remote `coordinator_register_descriptor_sql_template`.
- The registration template calls `ec_spire_register_remote_node_descriptor`
  with `decode('{remote_index_identity_hex}', 'hex')`, where the placeholder is
  filled from the remote endpoint's `profile_fingerprint`.
- The template also carries active epoch values into `last_served_epoch` and
  `min_retained_epoch`, and uses the remote endpoint `extension_version`.
- Added SQL literal escaping tests for generated SQL fragments.

## Validation

See `artifacts/manifest.md`.

- `cargo test -p ecaz-cli commands::corpus::load::tests`
- Result: 37 passed, 0 failed

## Scope Notes

This removes the prior fake-identity registration pattern from the generated
operator plan, but it still does not execute the remote identity query or apply
the coordinator registration automatically. The next slice should turn this
plan into an executable orchestration command or local 1+3 fixture.
