# PG18 Contributor Lookahead Owner Diagnostic

## Summary

This packet covers commit `60029a76d4be2afefff32b5dbcc6ddcfe222b7f0`.

The contributor diagnostic path now lets the elected visible tuple emitter
publish its prefetched next graph-traversal row as the visible owner snapshot
immediately after emitting a tuple. The publication is diagnostic-only and is
gated by `TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1`; the default elected
visible tuple emitter lane remains unchanged.

The new Ecaz Stats counter is:

- `Parallel Visible Owner Lookahead Publishes`

## Question

Packet 612 showed that dropping stable no-owner rows let contributors continue
working, but useful handoffs stayed at zero. One remaining uncertainty was
whether contributors were still racing a too-short visible-owner lifetime: the
elected visible emitter prefetched its next serial row after returning a tuple,
but did not publish that next owner until the next `amgettuple` call.

This packet tests whether publishing that prefetched lookahead owner during
contributor diagnostics removes `No Visible Owner` classifications and turns
contributor work into useful handoffs.

The measured fixture is:

- rows: 50000
- dimensions: 16
- randomized embeddings: true
- limit: 100
- ef_search: 500
- workers: 4

## Result

The default elected-emitter lane remains clean and does not publish lookahead
owners:

```text
[pg18-parallel] env=[]
[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500
Limit (actual time=14.465..15.493 rows=100.00 loops=1)
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
Parallel Contributor No Visible Owner Drops: 0
Parallel Visible Owner Lookahead Publishes: 0
Parallel Contributor Duplicate Retires: 0
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 0
Parallel Contributor Poll Limit: No Visible Owner: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane now sees the prefetched visible owner and no
longer classifies hidden rows as no-owner:

```text
[pg18-parallel] env=["TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC"]
[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500
Limit (actual time=35.199..36.988 rows=100.00 loops=1)
Bootstrap Expansions: 101
Elements Scored: 101
Heap TIDs Returned: 100
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 8
Parallel Contributor Publish: Duplicate Active: 8
Parallel Contributor Publish: Handoff Ready: 0
Parallel Contributor Publish: Ordered After Visible: 0
Parallel Contributor Publish: No Visible Owner: 0
Parallel Contributor No Visible Owner Drops: 0
Parallel Visible Owner Lookahead Publishes: 100
Parallel Contributor Duplicate Retires: 4
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 4
Parallel Contributor Poll Limit: Duplicate Active: 4
Parallel Contributor Poll Limit: No Visible Owner: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

## Interpretation

Lookahead visible-owner publication removes the owner-lifetime visibility
problem: `Parallel Contributor Publish: No Visible Owner`,
`Parallel Contributor No Visible Owner Drops`, and
`Parallel Contributor Poll Limit: No Visible Owner` all fall to zero in the
diagnostic lane while the default lane remains unchanged.

Useful handoffs remain zero:

- no foreign-selected handoffs,
- no foreign-head handoffs,
- no handoff-ready hidden publishes,
- no ordered-after-visible hidden publishes.

The remaining contributor work is duplicate-active against the elected
emitter's serial prefix. The next performance path should therefore target
disjoint/shared frontier sequencing rather than only extending visible-owner
lifetime.

## Artifacts

- `artifacts/pg18-parallel-50k-dim16-lookahead-default.log`
- `artifacts/pg18-parallel-50k-dim16-lookahead-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test explain_counters --lib`
- `cargo test visible_emitter_lookahead --lib`
- `cargo test contributor --lib`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-lookahead-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-lookahead-diagnostic.log`
- `cargo pgrx test pg18`
- `git diff --check`
