# PG18 Contributor No-Owner Drop Diagnostic

## Summary

This packet covers commit `9de975894acf48f407d93dbd7aed0f662e898028`.

The contributor diagnostic path now drops a hidden contributor row after it
repeatedly classifies as `NoVisibleOwner`. This is diagnostic-only behavior on
the non-emitting contributor path; the default elected visible tuple emitter
lane remains unchanged.

The new Ecaz Stats counter is:

- `Parallel Contributor No Visible Owner Drops`

## Question

Packet 611 showed that the high-dimensional fixture still left contributors
stuck behind hidden rows whose publish classifications were `No Visible Owner`.
This packet tests whether those orphan hidden rows were simply monopolizing the
contributor loop and preventing later contributor work from reaching a useful
handoff state.

The measured fixture is:

- rows: 50000
- dimensions: 16
- randomized embeddings: true
- limit: 100
- ef_search: 500
- workers: 4

## Result

The default elected-emitter lane remains clean:

```text
[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500
Limit (actual time=13.316..14.487 rows=100.00 loops=1)
Bootstrap Expansions: 101
Elements Scored: 101
Heap TIDs Returned: 100
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Publish: No Visible Owner: 0
Parallel Contributor No Visible Owner Drops: 0
Parallel Contributor Output Limit Exits: 0
Parallel Contributor Poll Limit Exits: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane now continues through orphan no-owner hidden
rows instead of timing out at the drain poll limit:

```text
[pg18-parallel] rows=50000 workers=4 dimensions=16 randomized_embeddings=true limit=100 ef_search=500
Limit (actual time=41.710..43.188 rows=100.00 loops=1)
Bootstrap Expansions: 101
Elements Scored: 101
Heap TIDs Returned: 100
Parallel Handoffs: Foreign Selected: 0
Parallel Handoffs: Foreign Head: 0
Parallel Contributor Hidden Publishes: 260
Parallel Contributor Publish: Duplicate Active: 4
Parallel Contributor Publish: Handoff Ready: 0
Parallel Contributor Publish: Ordered After Visible: 0
Parallel Contributor Publish: No Visible Owner: 256
Parallel Contributor No Visible Owner Drops: 252
Parallel Contributor Duplicate Retires: 4
Parallel Contributor Output Limit Exits: 4
Parallel Contributor Poll Limit Exits: 0
Parallel Contributor Poll Limit: No Visible Owner: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

## Interpretation

The contributor loop was not blocked only by the first orphan hidden row.
Dropping stable no-owner rows lets contributors keep working: hidden publishes
increase from 8 in packet 611 to 260 here, and the diagnostic lane exits by the
contributor output limit instead of the poll limit.

Useful handoffs remain zero:

- no foreign-selected handoffs,
- no foreign-head handoffs,
- no handoff-ready hidden publishes,
- no ordered-after-visible hidden publishes.

The next performance path should therefore target shared work sequencing rather
than retrying the same hidden-row drain loop. The likely next slice is a
serial-rank/shared-frontier model, or a stronger visible-owner lifetime model
that lets contributor work become admissible before the visible emitter has
already advanced past the relevant serial prefix.

## Artifacts

- `artifacts/pg18-parallel-50k-dim16-no-owner-drop-default.log`
- `artifacts/pg18-parallel-50k-dim16-no-owner-drop-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test explain_counters --lib`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --log-output target/pg18-parallel-50k-dim16-no-owner-drop-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --rows 50000 --limit 100 --dimensions 16 --randomized-embeddings --ef-search 500 --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-50k-dim16-no-owner-drop-diagnostic.log`
- `cargo pgrx test pg18`
- `git diff --check`
