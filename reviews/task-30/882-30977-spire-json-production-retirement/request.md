# Review Request: SPIRE JSON Production Path Retirement

- coder: coder1
- code/tracker commit: `89c0611d23c2946d4a8b3a9a2236c7c1a76be17e`
- tracker row: Phase 12.2 JSON endpoint production-path retirement

## Scope

This packet closes the Phase 12.2 row to remove the compatibility JSON tuple
payload endpoint from the production coordinator dispatch path.

The change keeps the legacy SQL endpoint
`ec_spire_remote_search_tuple_payload(...)` and the
`ec_spire.remote_tuple_transport = json_tuple_payload_v1` GUC value available
for compatibility and measurement, but production tuple-payload dispatch no
longer selects the JSON SQL template. When a tuple-payload read reaches a remote
endpoint that does not prefer ready `pg_binary_attr_v1` typed transport, the
coordinator now fails closed with the existing endpoint-identity failure
category.

## Changes

- Removed the production JSON tuple-payload SQL template from
  `remote_candidates.rs`.
- Added `remote_tuple_payload_production_sql(...)` so the production dispatch
  branch can only return the typed tuple payload SQL template.
- Added a unit test proving typed-ready endpoints select the typed template and
  JSON-default or missing-capability endpoints fail closed.
- Updated the `ec_spire.remote_tuple_transport` help text to stop advertising
  JSON as a production dispatch path.
- Updated the Phase 12 tracker to record that `serde_json` remains required by
  other runtime paths, including DML CustomScan JSON update payload handling and
  JSONB manifest surfaces.

## Validation

- `cargo test remote_tuple_transport`
  - passed 3 tests:
    - `remote_tuple_transport_auto_uses_endpoint_default`
    - `remote_tuple_transport_session_override_keeps_capability_gate`
    - `remote_tuple_payload_production_sql_requires_typed_transport`
- `cargo fmt --check`
  - passed with the repo's existing stable-rustfmt warnings for unstable
    `imports_granularity` and `group_imports`.
- `git diff --check`
  - passed.

## Reviewer Focus

- Does the fail-closed endpoint-identity category remain the right production
  status for typed tuple transport not being selectable?
- Is keeping the legacy JSON SQL endpoint/GUC value for compatibility and
  measurement acceptable now that production dispatch cannot choose it?
- Is the `serde_json` retention note sufficient for the tracker row's
  conditional dependency cleanup clause?
