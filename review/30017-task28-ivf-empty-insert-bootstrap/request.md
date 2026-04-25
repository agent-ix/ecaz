# Review Request: Task 28 IVF Empty Insert Bootstrap

Scope: Phase 5 live-insert checkpoint. The first `aminsert` into an empty IVF
index now bootstraps trained metadata, centroids, directory entries, and the
first posting from the inserted row.

Task: `plan/tasks/28-ivf-access-method.md` Phase 5

Branch: `task28-ivf`

Head SHA: `6b55219ab30c901f9e972181044e81b393b0a3ad`

Owner: coder2

Files:

- `src/am/ec_ivf/build.rs`
- `src/am/ec_ivf/insert.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`

Validation:

- `cargo check --no-default-features --features pg18 --tests`
- `cargo pgrx test pg18 test_ec_ivf_insert_bootstraps_empty_index`
- `cargo pgrx test pg18 test_ec_ivf_insert_appends_posting_and_updates_stats`
- `git diff --check`

Validation notes:

- Validation was PG18-only per the current AGENTS policy and the explicit user
  direction to test with PG18.
- The new PG test was run against PostgreSQL 18.3 through pgrx.
- No measurement claim is made in this packet.

## Summary

This slice closes empty-index first insert for Phase 5:

- Exposes the staged IVF build plan/flush path inside the IVF module so insert
  can reuse the same single-row training/storage path.
- Converts empty-index `aminsert` into a one-row build bootstrap that preserves
  persisted reloptions from metadata.
- Verifies the bootstrapped index has trained dimensions, centroids, directory
  entries, one live row, and scan-reachable output.
- Keeps the non-empty append/stat insert test green after adding the bootstrap
  branch.
- Updates the task plan so remaining Phase 5 work is duplicate handling,
  fuller quantizer-shape validation, and concurrency coverage.

## Review Focus

Please review for:

- Whether reusing the staged build planner is the right empty-index bootstrap
  contract, especially for explicit `nlists > 1` with a single inserted row.
- Whether `inserted_since_build = 0` is the correct drift value for a first row
  that bootstraps trained metadata rather than appending after build.
- Whether the reloptions reconstructed from metadata are sufficient, or whether
  insert should still consult relation options for empty indexes.
- Whether this path needs a narrower metadata-page lock before concurrent
  insert coverage lands.

## Non-Goals

This packet does not implement duplicate heap-TID coalescing/rejection,
concurrent insert stress coverage, vacuum cleanup, planner costing, heap/source
rerank, or measurement gates.
