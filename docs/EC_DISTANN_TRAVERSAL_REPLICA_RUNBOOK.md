# ec_distann traversal replica runbook

The coordinator traversal replica is an optional, explicit-build acceleration
object for one Published physical generation. It is not an owner generation,
payload store, mutation authority, or backup. Owner traversal remains the
correctness fallback.

## Deployment envelope

- Use one authoritative coordinator and one control index per source table.
- Treat the indexed generation as read-mostly. Construction is never automatic.
- Budget coordinator disk and WAL before building. Task 198 measured about
  1.660 GB of replica storage, 1.926 GB of WAL, and 52 seconds of build time at
  100k rows. The Task 199 release packet is the source of truth for promoted
  measurements.
- The extension-owner control connection must pass its bounded loopback
  authentication preflight. It uses 5-second connection, statement, and lock
  timeouts. If password authentication is required, configure the absolute,
  owner-only `replica_control_password_file`.
- Normal querying roles do not need direct catalog grants. Replica lookups run
  under the extension owner with a fixed restricted search path.

## Build and inspect

Run the control preflight before a build:

```sql
SELECT ec_distann_traversal_replica_control_preflight('my_index'::regclass);
```

Build the active Published generation explicitly:

```sql
SELECT encode(
    ec_distann_build_traversal_replica('my_index'::regclass),
    'hex'
);
```

Inspect identity, state, counts, byte sizes, build duration, last error, pins,
and reclaim eligibility:

```sql
SELECT *
FROM ec_distann_traversal_replica_status('my_index'::regclass);
```

Only a complete `Ready` row matching the active epoch is eligible. Missing,
Building, Stale, Retiring, malformed, identity-mismatched, relation-locked, or
unreadable images use owner traversal. A failure to durably demote a bad Ready
image emits a bounded warning but does not prevent owner fallback.

## Transaction isolation and writes

REPEATABLE READ and SERIALIZABLE reads bypass the replica and use owner
traversal. This fallback is read-only and does not demote a healthy Ready image.

Writes through an ec_distann index require READ COMMITTED. A stronger-isolation
snapshot cannot safely prove that a concurrent build did not just commit
Ready, so such writes fail before lookup or dispatch with:

- SQLSTATE `25001`
- token `EC_TRANSACTION_ISOLATION`

The first READ COMMITTED mutation that sees Ready durably changes it to Stale,
dispatches no owner mutation, and returns:

- SQLSTATE `40001`
- token `EC_REPLICA_INVALIDATED`

The current Published distributed-control index then retains its pre-existing
fail-closed `EC_GENERATION_MISSING` mutation posture. Task 199 does not promise
that retrying the statement commits an owner write.

VACUUM has no application retry loop. If it encounters a Ready image, it
durably marks the image Stale, emits `EC_REPLICA_INVALIDATED` as a warning, and
continues ordinary owner-index maintenance.

Replica-presence lookup errors fail ec_distann mutation guards closed, including
for an index that has never had a replica. This is deliberate: an unavailable
catalog cannot safely prove absence. Repair catalog/control availability before
retrying DML.

## Retire, reclaim, and rebuild

Retire and reclaim before rebuilding a Stale image for the same generation:

```sql
SELECT ec_distann_retire_traversal_replica('my_index'::regclass);
SELECT ec_distann_reclaim_traversal_replica('my_index'::regclass);
```

Reclaim returns false while a pinned scan still uses the immutable image.
Retry after pins drain. Both operations are idempotent.

If the dedicated invalidation connection is broken, repair authentication and
run:

```sql
SELECT ec_distann_recover_traversal_replica_invalidation(
    'my_index'::regclass
);
```

Then retire/reclaim or rebuild as appropriate. Superseded epochs are retired
automatically and can be reclaimed after their pins drain.
