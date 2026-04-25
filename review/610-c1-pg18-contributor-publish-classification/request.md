# PG18 Contributor Publish Classification

## Summary

This packet covers commit `e27e7f4e8c62f56ccbb94b46312f5b0fe1cd3fd0`.

Packet 607 showed that contributors exit with `Poll Limit: No Visible Owner`,
which only describes the final state after the drain window has closed. This
checkpoint adds publish-time classification counters for each hidden contributor
row, using the same hidden-drain classifier immediately after the row is staged.

New Ecaz Stats counters:

- `Parallel Contributor Publish: Missing Hidden`
- `Parallel Contributor Publish: Duplicate Active`
- `Parallel Contributor Publish: Handoff Ready`
- `Parallel Contributor Publish: Ordered After Visible`
- `Parallel Contributor Publish: No Visible Owner`

The change is diagnostic-only. It bumps the parallel DSM version from 18 to 19,
records the new shared counters, folds them into the elected emitter's Ecaz
Stats, and extends unit coverage for local and shared publish classification.

## Result

Default LIMIT 64 remains clean with all contributor counters at zero:

```text
Limit (actual time=15.243..15.883 rows=64.00 loops=1)
Bootstrap Expansions: 65
Elements Scored: 65
Heap TIDs Returned: 64
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Publish: No Visible Owner: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Contributor diagnostic LIMIT 64 now shows the publish-time split:

```text
Limit (actual time=33.829..35.093 rows=64.00 loops=1)
Bootstrap Expansions: 65
Elements Scored: 65
Heap TIDs Returned: 64
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 8
Parallel Contributor Publish: Missing Hidden: 0
Parallel Contributor Publish: Duplicate Active: 4
Parallel Contributor Publish: Handoff Ready: 0
Parallel Contributor Publish: Ordered After Visible: 0
Parallel Contributor Publish: No Visible Owner: 4
Parallel Contributor Duplicate Retires: 4
Parallel Contributor Poll Limit Exits: 4
Parallel Contributor Poll Limit: No Visible Owner: 4
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

## Interpretation

The 8 hidden publishes split into 4 duplicate-active rows and 4 rows that already
have no selected/admitted visible owner at publish time. There are still no
handoff-ready or ordered-after-visible rows.

That narrows the next performance path: current contributors are not losing an
ordering predicate while the emitter is active. Half of their published work is
duplicate work, and the non-duplicate half arrives after the elected emitter's
visible owner window has already closed.

## Artifacts

- `artifacts/pg18-parallel-limit64-publish-classification-default.log`
- `artifacts/pg18-parallel-limit64-publish-classification-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test explain_counters --lib`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --limit 64 --log-output target/pg18-parallel-limit64-publish-classification-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --limit 64 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-limit64-publish-classification-diagnostic.log`
- `cargo pgrx test pg18`
- `git diff --check`
