# PG18 Contributor Exit Counters

## Summary

This packet covers commit `2e6198badc3201fd961b133b1fc359243039c453`.

The PG18 contributor diagnostic path now reports why non-emitting contributors
stop after publishing hidden output. Two shared DSM counters were added and
folded into the elected emitter's Ecaz Stats output:

- `Parallel Contributor Output Limit Exits`
- `Parallel Contributor Poll Limit Exits`

This is diagnostic visibility, not a speedup. The diagnostic lane now confirms
the current performance blocker: workers publish hidden rows, retire some
duplicates, then all four non-emitting contributors exit on the drain-poll
limit before any useful handoff is consumed.

## Result

The default elected-emitter lane still passes and remains contributor-neutral:

```text
Limit (actual time=13.809..15.187 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Duplicate Retires: 0
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane exposes the poll-limit exit behavior:

```text
Limit (actual time=33.622..35.003 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 8
Parallel Contributor Duplicate Retires: 4
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 4
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Interpretation: there is still a path forward, but it is not just "make the
contributors run longer." The next performance slice needs rank-aware distinct
contribution or an emitter-drain change that turns hidden published rows into
useful handoffs without letting later exact-smaller rows overtake the serial
HNSW-rank stream.

## Artifacts

- `artifacts/pg18-parallel-contributor-exit-counters-default.log`
- `artifacts/pg18-parallel-contributor-exit-counters-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test explain_counters --lib`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-exit-counters-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-exit-counters-diagnostic.log`
- `cargo pgrx test pg18`
- `git diff --check`
