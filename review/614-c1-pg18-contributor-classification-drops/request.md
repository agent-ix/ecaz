# PG18 Contributor Classification Drops

## Summary

This packet covers commit `7e3c45915ff977ed63c1e3bcbeae52e500480154`.

The contributor diagnostic path now drops hidden contributor rows that remain
stuck in classifications known not to produce an immediately safe handoff:

- `DuplicateActive`
- `OrderedAfterVisible`

These drops are diagnostic-only behavior reached by the existing
`TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1` path. The default elected
visible tuple emitter lane remains unchanged.

The new Ecaz Stats counters are:

- `Parallel Contributor Duplicate Active Drops`
- `Parallel Contributor Ordered After Visible Drops`

## Question

Packets 612 and 613 removed the obvious owner-lifetime blockers, but
contributors still exited without useful handoffs. Packet 613 showed the next
stable blocker was duplicate-active against the elected serial prefix.

This packet tests whether dropping duplicate-active rows, and then the
subsequent ordered-after-visible rows, lets contributors advance far enough to
produce handoff-ready work.

The measured fixture is:

- rows: 50000
- dimensions: 16
- randomized embeddings: true
- limit: 100
- ef_search: 500
- workers: 4

## Result

The default elected-emitter lane remains clean and contributor-neutral:

```text
[pg18-parallel] env=[]
[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500
Limit (actual time=14.432..16.167 rows=100.00 loops=1)
Bootstrap Expansions: 101
Elements Scored: 101
Heap TIDs Returned: 100
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Publish: Duplicate Active: 0
Parallel Contributor Publish: Handoff Ready: 0
Parallel Contributor Publish: Ordered After Visible: 0
Parallel Contributor Publish: No Visible Owner: 0
Parallel Contributor Duplicate Active Drops: 0
Parallel Contributor Ordered After Visible Drops: 0
Parallel Visible Owner Lookahead Publishes: 0
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane advances past the previous poll-limit blockers:

```text
[pg18-parallel] env=["TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC"]
[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500
Limit (actual time=41.957..43.911 rows=100.00 loops=1)
Bootstrap Expansions: 101
Elements Scored: 101
Heap TIDs Returned: 100
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 260
Parallel Contributor Publish: Duplicate Active: 8
Parallel Contributor Publish: Handoff Ready: 0
Parallel Contributor Publish: Ordered After Visible: 252
Parallel Contributor Publish: No Visible Owner: 0
Parallel Contributor Duplicate Active Drops: 4
Parallel Contributor Ordered After Visible Drops: 248
Parallel Visible Owner Lookahead Publishes: 100
Parallel Contributor Duplicate Retires: 4
Parallel Contributor Output Limit Exits: 4
Parallel Contributor Poll Limit Exits: 0
Parallel Contributor Poll Limit: Duplicate Active: 0
Parallel Contributor Poll Limit: Ordered After Visible: 0
Parallel Contributor Poll Limit: No Visible Owner: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

## Interpretation

The diagnostic drops remove the drain-loop obstruction: contributors no longer
exit on the poll limit, and they publish 260 hidden rows before leaving via the
output limit.

Useful handoffs still remain zero:

- no foreign-selected handoffs,
- no foreign-head handoffs,
- no handoff-ready hidden publishes.

The new dominant classification is `OrderedAfterVisible`, with 248 diagnostic
drops. That means contributor work is advancing behind or after the elected
visible serial prefix rather than producing distinct rows that can safely
precede it. The next performance path should target rank-aware distinct
frontier ownership or shared frontier sequencing, not more drain-loop retries.

## Artifacts

- `artifacts/pg18-parallel-50k-dim16-classification-drop-default.log`
- `artifacts/pg18-parallel-50k-dim16-classification-drop-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test explain_counters --lib`
- `cargo test contributor --lib`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-classification-drop-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-classification-drop-diagnostic.log`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18`
- `git diff --check`
