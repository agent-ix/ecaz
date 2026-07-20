# Review request: physical `DROP EXTENSION` cleanup drill

## Scope

Please review the deferred Task 179 closeout drill for dropping `ecaz` while a
distributed control index owns both an active `Published` generation and an
unpublished `Ready` generation on three real PG18 instances.

Code checkpoints:

- `a37e3f56f7ae7a3cecf9e1d7cef4c9d5868fd4df` adds the opt-in external
  lifecycle drill and `ecaz bench suite` surface.
- `0e8e8f6e823cae06b53a3dc0157c631a84dabb6d` strengthens the precondition to
  require one `Ready` and one `Published` generation per owner before the
  extension is dropped.

This is a CLI fixture and evidence checkpoint. It does not change the
production extension; `src/` and `sql/` are byte-unchanged from the measured
extension source `9387f72b3` through this packet's head.

## What the drill proves

The canonical one-step suite:

1. starts three PG18 instances with `shared_preload_libraries=ecaz`;
2. builds, publishes, serves, and remotely materializes a 90-row physical
   generation;
3. builds epoch 2 to `Ready` without deciding or publishing it;
4. verifies every owner has one `Ready`, one `Published`, and six `_ecdz_*`
   physical relations;
5. executes external `DROP EXTENSION ecaz CASCADE` on every instance; and
6. proves each instance has zero installed `ecaz` extension rows, zero
   `_ecdz_*` relations, and can create/insert/read an ordinary table while the
   library remains preloaded.

All three owners pass:

```text
node=1 Ready=1 Published=1 hidden=6 -> extension=0 hidden=0 DML rows=1
node=2 Ready=1 Published=1 hidden=6 -> extension=0 hidden=0 DML rows=1
node=3 Ready=1 Published=1 hidden=6 -> extension=0 hidden=0 DML rows=1
```

The suite completed 1/1 steps with 2/2 thresholds, zero missing artifacts, and
zero stale artifacts. Ready/Published topology is exact and disjoint at
33/24/33 records with zero non-owned rows and zero orphans; both remote owners
also pass frozen-row materialization before the destructive drill.

## Reviewer focus

1. Confirm that the second generation is genuinely unpublished (`Ready`) and
   coexists with the active `Published` epoch when `DROP EXTENSION` begins.
2. Confirm that checking `_ecdz_*` relations after the extension disappears
   closes the event-trigger/internal-dependency cleanup concern from packet
   003, rather than merely proving the SQL catalogs were themselves dropped.
3. Confirm that the post-drop DML probe exercises the preloaded-hook
   passthrough path after the extension catalogs no longer exist.
4. Confirm this packet discharges the cleanup drill named by Task 179's sole
   closeout packet prerequisite; it does not by itself mark Task 179 complete.

## Validation

- exact-SHA release runner build: pass;
- focused suite parser test: 1 passed, 0 failed;
- `cargo check -p ecaz-cli`: pass with one pre-existing unrelated dead-code
  warning;
- canonical PG18 suite: 1/1 succeeded, 2/2 thresholds passed.

PostgreSQL server logs and the isolated run directory are not committed.
