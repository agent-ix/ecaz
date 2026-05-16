# Review Request: SPIRE Local Store Diagnostic Unit

Code checkpoint: `a5322647` (`Close SPIRE local-store diagnostic key checks`)

## Scope

- Advances Phase 12.8 local multi-store readiness by marking the
  `(node_id, local_store_id)` scheduling/diagnostic unit and bounded local
  store lookup rows complete.
- Strengthens the PG18 two-store SQL VACUUM fixture to assert:
  - placement diagnostics expose two distinct `(node_id, local_store_id)` keys;
  - placement diagnostics expose two distinct
    `(node_id, local_store_id, store_relid)` keys;
  - scan placement diagnostics expose only local-node
    `(node_id, local_store_id)` groups for the query-selected stores;
  - post-insert/post-delete VACUUM still leaves no active delta object or delta
    assignment debt.
- Updates `docs/SPIRE_DIAGNOSTICS.md` to state the placement and scan-placement
  diagnostic grouping keys and the bounded local-store lookup maps.
- Updates the Phase 12 tracker with evidence from this fixture plus the
  earlier packet `30678` indexed-lookup implementation.

## Validation

- `git diff --check a5322647^ a5322647`
- `cargo fmt --check`
- `cargo pgrx test pg18 test_pg18_ec_spire_multistore_sql_vacuum_routes_local_stores`

Packet-local logs are under `artifacts/`; see `artifacts/manifest.md` for
commands and key result lines.

## Review Focus

- Confirm it is acceptable for this fixture to validate post-VACUUM state
  through SPIRE diagnostics rather than broad heap SELECTs, which now trip the
  current DML frontdoor fail-closed policy for ec_spire-indexed coordinator
  tables.
- Confirm the Phase 12.8 checklist wording is scoped to the implemented
  diagnostic and indexed-lookup contracts, not to the still-open read-overlap
  harness or multi-NVMe performance evidence.
