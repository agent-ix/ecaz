# PG18 Contributor Shared Counters

## Summary

This packet covers commit `368a31ef56a494c24e57b3eab715f16d986963e8`.

The hidden contributor publish and duplicate-retire counters now live in shared
parallel scan state as DSM atomics. Non-emitting contributors increment the
shared counters when they publish hidden output or retire a hidden duplicate,
and the elected visible emitter folds those shared totals into its backend-local
Ecaz Stats snapshot during `amgettuple`, while the shared mapping is still
valid for runtime access.

This is diagnostic visibility, not a speedup. The diagnostic lane now proves
that non-emitting workers are publishing hidden rows, but the fixture still
emits the same 16 rows as serial and reports zero useful foreign handoffs.

## Result

The default elected-emitter lane still passes and remains contributor-neutral:

```text
Limit (actual time=13.478..14.567 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Duplicate Retires: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane now exposes hidden contributor work in the
elected emitter's Ecaz Stats output:

```text
Limit (actual time=33.686..34.986 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Contributor Hidden Publishes: 8
Parallel Contributor Duplicate Retires: 4
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Interpretation: the non-emitting contributor path is active under the diagnostic
env and the elected emitter can report shared contributor work without reading
DSM from the explain hook. The next performance step is still a rank-aware
distinct contribution path that turns hidden publishes into useful handoffs.

## Artifacts

- `artifacts/pg18-parallel-contributor-shared-counters-default.log`
- `artifacts/pg18-parallel-contributor-shared-counters-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-shared-counters-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-shared-counters-diagnostic.log`
- `git diff --check`
