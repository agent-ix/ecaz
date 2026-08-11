# Task 167 checkpoint: owner-persisted prepared-write intent

This checkpoint extends the physical-generation DML path with owner-side
durable intent tracking for prepared remote inserts and backlink amendments.
Intent rows are created and advanced over the owner connection, so a
coordinator abort cannot roll them back as part of the user transaction.
`prepare_acked` remains unresolved until the commit/abort callback records the
final state; the explicit reaper fences by index, owner, epoch, and coordinator
XID before resolving orphaned prepared transactions.

Also included:

- exact robust-prune backlink amendments using the target source vector and
  materialized remote neighbor vectors;
- affected-row checking on backlink rewrites;
- UPDATE identity classification and retained current-version replacement;
- parser tests for prepared-transaction identity fencing.

Validation:

- `cargo check --no-default-features --features pg18` — passed at `cbd14ab66`.
- `cargo test --no-default-features --features pg18 --lib ec_distann::remote_transport` —
  completed after rebuild; the shared pgrx integration runner remains separate
  and was not used as closeout evidence.

This is not a closeout request. TC-043/NFR-020 physical drills, routed
tombstone delete, concurrent insert/query evidence, and the required
10k/50k/100k suite A/B evidence remain open.

See `artifacts/manifest.md` and `artifacts/validation.log`.
