# Review request: inactive durable-gate fast path

## Scope

Please review commit `a4d374c2f294dc209b1b0f499bd527e52b375b06`,
which adds a transactionally invalidated backend-local negative cache for the
database-wide absence of active DistANN build registrations.

A preliminary A/B exposed material installed/no-gate overhead and motivated
this checkpoint. That diagnostic run was overwritten by the canonical
post-fix rerun and is not durable review evidence; packet 057 owns the
traceable exact-SHA result used for closeout.

## Design

- Only the database-wide **no active gate** result is cached. A positive state
  is never enough to answer a relation-specific check, so active-gate DML still
  performs the durable OID/UUID/source validation.
- An `AFTER ... FOR EACH STATEMENT` trigger on
  `ec_distann_build_registration` covers INSERT, UPDATE, DELETE, and TRUNCATE.
  It clears the mutating backend immediately and calls PostgreSQL's
  `CacheInvalidateRelcacheByRelid`.
- PostgreSQL publishes that relcache invalidation to other backends at commit;
  rollback does not create a false positive. The trigger is `ENABLE ALWAYS` so
  replica session role cannot suppress invalidation.
- Every backend registers a relcache callback during preload. It clears the
  negative conservatively on any relcache event, avoiding a cached table OID
  across DROP/CREATE EXTENSION or other DDL.
- The first post-invalidation DML probes the global durable registration state.
  If no validated active registration exists, subsequent DML pays only the
  installed-extension probe plus the backend-local bit check.
- Begin-build already acquires its conflicting source lock before inserting the
  registration. This closes the boundary where source DML could otherwise
  begin between registration commit and invalidation processing.

## Cross-backend regression

The focused existing lifecycle test is specifically sensitive to stale
negatives:

1. a separate backend prepares and executes a source INSERT before any gate,
   warming the negative cache;
2. the owner backend commits the first durable registration;
3. the warmed backend reuses its cached generic plan; and
4. execution must fail `EC_BUILD_STATE`, not reach `aminsert` and fail
   `EC_GENERATION_MISSING`.

That path passes, together with the existing source/utility matrix, ATTACH
PARTITION, indirect TRUNCATE CASCADE, EXPLAIN-only, savepoint, and unrelated
table controls.

## Reviewer focus

1. Confirm trigger invalidation is transactional for remote backends and
   immediate for the mutating backend.
2. Confirm the source lock acquired before registration closes the commit /
   command-boundary race for source DML.
3. Confirm no positive or relation-specific result is cached and that extension
   DROP/recreate cannot retain an unsafe negative.
4. Confirm `ENABLE ALWAYS` plus statement coverage includes every durable
   registration mutation surface, including direct operator SQL.

## Validation

- `cargo check --no-default-features --features pg18`: pass.
- focused live PG18 lifecycle regression: 1 passed, 0 failed, 2,507 filtered.

Complete exact-SHA logs are packet-local under `artifacts/`.
