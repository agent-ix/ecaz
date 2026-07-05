# Review Request: SPIRE remote-search final fixture split

## Summary

This checkpoint moves the remaining remote-search-adjacent SPIRE fixtures out
of `src/tests/mod.rs` and into `src/tests/remote_search.rs`.

Moved fixtures:

- `test_ec_spire_remote_search_local_heap_resolution_plan`
- `test_ec_spire_remote_search_degraded_stale_leaf`
- `test_ec_spire_reaper_resolves_lost_prepare_ack_fixture`
- `test_ec_spire_remote_pk_select_isolation_contract_sql`

`unsafe fn analyzed_query` remains in `src/tests/mod.rs` because
`src/tests/dml_frontdoor.rs` still uses it through the existing include-based
test module structure. The next module-tree cleanup slice can move or
deduplicate that helper when the DML include boundary is addressed.

`src/tests/remote_search.rs` is now 12,245 lines. That is intentionally called
out for review; packet 31017 scoped the hard 2,500-line cap to
`src/am/ec_spire/`, not `src/tests/`, and this slice preserves fixture behavior
while removing the last remote-search bodies from the umbrella module.

Code checkpoint: `141ab3e64fe1f02a6487e7febaea41fe77fcbb20`

## Validation

- `cargo fmt --check`
- `git diff --check`
- location check confirms the four moved fixtures now resolve in
  `src/tests/remote_search.rs`
- focused PG18 checks:
  - `test_ec_spire_remote_search_local_heap_resolution_plan`: passed
  - `test_ec_spire_remote_search_degraded_stale_leaf`: passed
  - `test_ec_spire_reaper_resolves_lost_prepare_ack_fixture`: passed
  - `test_ec_spire_remote_pk_select_isolation_contract_sql`: passed

Raw logs and command metadata are in `artifacts/manifest.md`.

## Reviewer Focus

- Confirm the move is behavior-preserving and no helper/import dependency was
  silently dropped.
- Confirm leaving `analyzed_query` in `mod.rs` is acceptable for this narrow
  remote-search fixture split.
- Confirm the tracker update matches the remaining source state: the only
  `test_ec_spire_*` bodies still directly in `src/tests/mod.rs` are the
  relation/manifest storage roundtrip fixtures.
