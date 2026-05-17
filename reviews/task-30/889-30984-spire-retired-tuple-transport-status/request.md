# Review Request: SPIRE Retired Tuple Transport Status

## Scope

Please review commit `7bf90533314c5b828e0265856ee10af03bc7af72`.

This addresses Phase 12a.5 from
`plan/tasks/task30-phase12a-spire-readiness-followups.md`: legacy JSON tuple
transport should fail with a specific operator category instead of the generic
endpoint-identity mismatch bucket.

## Changes

- Adds `tuple_transport_retired` as a distinct production failure category.
- Returns that category from `remote_tuple_payload_production_sql(...)` whenever
  the endpoint identity is valid but production cannot select
  `pg_binary_attr_v1`.
- Adds `first_skip_hint` to
  `ec_spire_remote_search_degraded_skip_report(...)`, with an actionable
  `pg_binary_attr_v1` upgrade/descriptor-refresh hint for this category.
- Updates the production fault matrix, runbook, diagnostics table, and Phase
  12a tracker.

## Validation

```sh
cargo test tuple_transport --lib
cargo test production_fault_matrix_covers_required_categories --lib
cargo test degraded_skip_report --lib
cargo fmt --check
git diff --check
```

`cargo fmt --check` reports the repository's existing stable-rustfmt warnings
for unstable `imports_granularity` / `group_imports`, then exits successfully.

## Review Ask

Confirm the new category is reserved for valid endpoint identities with retired
or unavailable production tuple transport, while
`endpoint_identity_mismatch` remains reserved for genuine identity mismatch
shapes.
