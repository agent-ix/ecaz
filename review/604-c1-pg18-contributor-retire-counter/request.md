# PG18 Contributor Duplicate Retire Counter

## Summary

This packet covers commit `f6d426ee93de3f6dfd206d290ae5d9b393b513b0`.

The obsolete hidden contributor retirement path now records an Ecaz Stats
counter named `Parallel Contributor Duplicate Retires`. The counter increments
when a non-emitting contributor retires a hidden row because its next heap TID
has already been emitted by the shared coordinator or another worker snapshot.

This is diagnostic visibility, not a performance improvement. The live PG18
fixture still reports zero duplicate retires and zero useful foreign handoffs,
so useful parallel speedup still depends on rank-aware distinct contribution.

## Result

The default elected-emitter lane still passes and exposes the new counter:

```text
Limit (actual time=14.093..15.152 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Duplicate Retires: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane also preserves serial equivalence and exposes
the counter, but remains counter-neutral on this fixture:

```text
Limit (actual time=34.083..34.948 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Duplicate Retires: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Interpretation: the retire path is now visible in Ecaz Stats and covered by a
unit assertion, but the current fixture still does not exercise useful
contributor work. The next performance path remains rank-aware distinct
contribution or an exact-key contract change.

## Artifacts

- `artifacts/pg18-parallel-contributor-retire-counter-default.log`
- `artifacts/pg18-parallel-contributor-retire-counter-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test retire_obsolete_non_emitting_parallel_contributor_output --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-retire-counter-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-retire-counter-diagnostic.log`
- `git diff --check`
