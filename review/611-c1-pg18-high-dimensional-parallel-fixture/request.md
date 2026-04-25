# PG18 High-Dimensional Parallel Fixture

## Summary

This packet covers commit `8ae8fe4a0e2a0b8446dd75b88cf0a696183d4bbc`.

The checkpoint adds `pg18-parallel-scan` fixture controls to `ecaz-cli`:

- `--dimensions`
- `--randomized-embeddings`

The default 4D deterministic fixture remains unchanged. The new controls allow
the same PG18 planner-visible parallel scan harness to generate higher
dimensional deterministic pseudo-random embeddings and queries without ad hoc
SQL scripts.

## Question

Packet 609 suggested that the previous 4D fixture might be too easy: even at
larger row counts, the elected visible emitter still reaches the full serial
prefix before hidden contributor rows become useful.

This packet tests the suggested larger surface:

- rows: 50000
- dimensions: 16
- randomized embeddings: true
- limit: 100
- ef_search: 500
- workers: 4

## Result

The default elected-emitter lane validates the new fixture and remains clean:

```text
[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500
Limit (actual time=15.930..16.944 rows=100.00 loops=1)
Bootstrap Expansions: 101
Elements Scored: 101
Heap TIDs Returned: 100
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Publish: No Visible Owner: 0
Parallel Contributor Poll Limit Exits: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane reproduces the prior classification:

```text
[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500
Limit (actual time=32.676..34.372 rows=100.00 loops=1)
Bootstrap Expansions: 101
Elements Scored: 101
Heap TIDs Returned: 100
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 8
Parallel Contributor Publish: Missing Hidden: 0
Parallel Contributor Publish: Duplicate Active: 4
Parallel Contributor Publish: Handoff Ready: 0
Parallel Contributor Publish: Ordered After Visible: 0
Parallel Contributor Publish: No Visible Owner: 4
Parallel Contributor Duplicate Retires: 4
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 4
Parallel Contributor Poll Limit: No Visible Owner: 4
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

## Interpretation

The high-dimensional randomized fixture is working and validates through the
same serial-vs-parallel comparison, but it does not unlock useful contributor
handoffs. The useful work remains zero:

- no foreign-selected handoffs,
- no foreign-head handoffs,
- no handoff-ready hidden publishes,
- no ordered-after-visible hidden publishes.

The prior `No Visible Owner` pattern is therefore not explained only by the old
4D fixture. The next performance path should target runtime coordination rather
than more fixture scaling: either rank-aware hidden admission, a longer-lived
visible-owner/drain window, or a serial-rank sequencer that can safely consume
contributor work without violating the serial prefix.

## Artifacts

- `artifacts/pg18-parallel-50k-dim16-random-default.log`
- `artifacts/pg18-parallel-50k-dim16-random-diagnostic.log`
- `artifacts/manifest.md`

## Validation

CLI checkpoint validation:

- `cargo fmt`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli -- -D warnings`

Measurement validation:

- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-random-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-random-diagnostic.log`
