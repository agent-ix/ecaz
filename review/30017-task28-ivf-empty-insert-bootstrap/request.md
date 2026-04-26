# Review Request: Task 28 IVF Empty Insert Bootstrap

Scope: Phase 5 live-insert checkpoint. The first `aminsert` into an empty IVF
index now bootstraps trained metadata, centroids, directory entries, and the
first posting from the inserted row.

Task: `plan/tasks/28-ivf-access-method.md` Phase 5

Branch: `task28-ivf`

Head SHA: `cb3c75ae71f6d897c43fd9781d24be647a2fae66`

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
- `cargo pgrx test pg18 test_pg18_ec_ivf_concurrent_empty_bootstrap_reachable`
- `cargo pgrx test pg18 test_pg18_ec_ivf_concurrent_same_list_inserts_remain_reachable`
- `cargo pgrx test pg18`
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
- Updates the task plan so the remaining IVF work is Phase 8 measurement and
  bench handoff.

Update after reviewer feedback: concurrent empty-index inserts now serialize
the empty-to-trained transition with a relation-level
`ShareUpdateExclusiveLock`, re-read metadata under that lock, bootstrap only if
the index is still empty, and route losing waiters through the normal trained
append path. Packet-local PG18 validation logs are stored under
`artifacts/`.

## Review Focus

Please review for:

- Whether reusing the staged build planner is the right empty-index bootstrap
  contract, especially for explicit `nlists > 1` with a single inserted row.
- Whether `inserted_since_build = 0` is the correct drift value for a first row
  that bootstraps trained metadata rather than appending after build.
- Whether the reloptions reconstructed from metadata are sufficient, or whether
  insert should still consult relation options for empty indexes.
- Whether the chosen relation-level `ShareUpdateExclusiveLock` is the right
  serialization contract for the empty-to-trained transition.

## Non-Goals

This packet does not make recall, latency, storage, WAL, or planner-cost
measurement claims. Those remain Phase 8 packet work.
