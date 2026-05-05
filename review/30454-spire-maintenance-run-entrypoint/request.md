# Review Request: SPIRE Maintenance Run Entrypoint

## Summary

Task 30 SPIRE Phase 2 now has the manual live maintenance scheduler entrypoint.

Changes:
- Expose `ec_spire_index_maintenance_run(index_oid)` through the SQL extension
  surface.
- Take the SPIRE publish lock, reload active manifests, collect leaf rows, and
  reselect the scheduled split/merge candidate under lock.
- Build relation execution input for merge replacements from the active
  snapshot.
- Build relation execution input for split replacements from live heap-source
  vectors, using the active heap snapshot and the existing dead-row filtering
  helper.
- Publish the selected scheduled replacement epoch through the existing
  relation publish helper and return the shared run-result row with
  `published = true`.

## Validation

- `cargo test maintenance_run --lib`
- `cargo fmt --check`
- `git diff --check`

## Notes

No focused PG18 runtime test was added in this slice; validation covered Rust
compile/unit surfaces and the lower-level publish/input helpers already covered
by prior slices.
No measurement claims.
