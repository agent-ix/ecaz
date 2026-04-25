# PG18 Contributor Drain Classification

## Summary

This packet covers commit `a6b72d66945afef24b51f0935a1461c3a4785051`.

The PG18 contributor diagnostic path now classifies each contributor poll-limit
exit against the shared coordinator state. The new Ecaz Stats counters are:

- `Parallel Contributor Poll Limit: Missing Hidden`
- `Parallel Contributor Poll Limit: Duplicate Active`
- `Parallel Contributor Poll Limit: Handoff Ready`
- `Parallel Contributor Poll Limit: Ordered After Visible`
- `Parallel Contributor Poll Limit: No Visible Owner`

This is diagnostic visibility only. It does not change the visible tuple stream.

## Result

The default elected-emitter lane remains contributor-neutral:

```text
Limit (actual time=13.761..14.902 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Duplicate Retires: 0
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 0
Parallel Contributor Poll Limit: Missing Hidden: 0
Parallel Contributor Poll Limit: Duplicate Active: 0
Parallel Contributor Poll Limit: Handoff Ready: 0
Parallel Contributor Poll Limit: Ordered After Visible: 0
Parallel Contributor Poll Limit: No Visible Owner: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane shows the current hidden rows are not timing out
because they are active duplicates, ordered after the visible row, or blocked by
admission. All four poll-limit exits happen after there is no selected or
admitted visible owner row left to compare against:

```text
Limit (actual time=33.791..34.848 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 8
Parallel Contributor Duplicate Retires: 4
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 4
Parallel Contributor Poll Limit: Missing Hidden: 0
Parallel Contributor Poll Limit: Duplicate Active: 0
Parallel Contributor Poll Limit: Handoff Ready: 0
Parallel Contributor Poll Limit: Ordered After Visible: 0
Parallel Contributor Poll Limit: No Visible Owner: 4
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Interpretation: the next performance slice should focus on the elected visible
emitter's drain timing/lifecycle. Contributors are still publishing hidden
rows, but by the time they give up, the shared coordinator no longer has a
visible selected/admitted owner row that can notice and drain those hidden rows.

## Artifacts

- `artifacts/pg18-parallel-contributor-drain-classification-default.log`
- `artifacts/pg18-parallel-contributor-drain-classification-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test explain_counters --lib`
- `cargo test classify_parallel_scan_contributor_hidden_drain --lib`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-drain-classification-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-drain-classification-diagnostic.log`
- `cargo pgrx test pg18`
- `git diff --check`
