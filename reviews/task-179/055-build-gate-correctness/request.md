# Review request: durable build-gate correctness follow-up

## Scope

Please review commit `4aa8817ce6bef68de54e8039972d2e10d0815b6a`,
which addresses packet 007's P2-A and P2-B findings plus the related
`TRUNCATE CASCADE`, EXPLAIN-only, and logical-replication documentation
follow-ups.

This packet closes the named correctness findings. It does **not** close the
build-gate topic or Task 179: the packet 007 P2-4 DML hot-path A/B remains
owed and will be submitted separately.

## Changes

- Every pre-`standard_ProcessUtility` `RangeVar` lookup now uses `NoLock`.
  PostgreSQL's statement-specific ownership callbacks and true lock levels
  therefore remain responsible for locking; an unprivileged failing utility
  command can no longer acquire this hook's prior escalated lock.
- `ALTER TABLE ... ATTACH PARTITION` inspects each `AT_AttachPartition`
  subcommand and gates the prospective child as well as the named parent.
- `TRUNCATE ... CASCADE` takes the global utility/build serialization lock and
  conservatively rejects while any source gate is active, covering relations
  reached indirectly through foreign keys.
- ExecutorStart skips enforcement for `EXEC_FLAG_EXPLAIN_ONLY`. EXPLAIN with
  ANALYZE remains gated because it executes the described DML.
- The logical-replication apply-worker bypass is now documented beside the
  executor hook: distributed sources must remain outside publications until a
  replication-specific gate exists.

## Live PG18 regression

The existing durable lifecycle test now creates two valid indirect rewrite
surfaces before registering the build:

1. a compatible partitioned parent for the standalone distributed source;
2. a referenced root table with a foreign key from the distributed source.

After the coordinator backend exits while the durable gate remains live, the
test proves:

- `ATTACH PARTITION <gated-source>` fails `EC_BUILD_STATE` and leaves no
  `pg_inherits` edge;
- `TRUNCATE <root> CASCADE` fails `EC_BUILD_STATE` and preserves the root row;
- `EXPLAIN (COSTS OFF) INSERT INTO <gated-source>` succeeds without execution;
- the prior cached-plan, DML, COPY, ALTER, DROP, CLUSTER, VACUUM FULL, REINDEX,
  global utility, savepoint, OID-reuse, and unrelated-table positive controls
  continue to pass.

Exact-SHA result: 1 passed, 0 failed, 2,507 filtered out.

## Reviewer focus

1. Confirm `NoLock` resolution removes the pre-permission and over-locking
   behavior without weakening the OID/UUID/source-identity revalidation in the
   mask helper.
2. Confirm `PartitionCmd::name` is the correct child identity for PG18
   `AT_AttachPartition` and is checked even when the parent is not gated.
3. Confirm the exclusive global serialization order for `TRUNCATE CASCADE`
   matches begin-build's shared side and prevents a registration/effect-set
   race.
4. Confirm `EXEC_FLAG_EXPLAIN_ONLY` distinguishes non-executing EXPLAIN from
   EXPLAIN ANALYZE.

## Validation

- `cargo check --no-default-features --features pg18`: pass.
- focused live `cargo pgrx test pg18
  test_distann_begin_build_competing_backend_busy --no-default-features
  --features pg18`: 1 passed, 0 failed.

Both complete logs are packet-local under `artifacts/`.
