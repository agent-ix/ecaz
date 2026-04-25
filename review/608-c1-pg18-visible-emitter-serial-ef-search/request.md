# PG18 Visible Emitter Serial ef_search

## Summary

This packet covers commit `74f7df0cc629673552c725848b24f91bbf54b681`.

Planner-visible PG18 parallel scans currently expose one elected visible tuple
emitter so the output stream stays serial-equivalent. The prior runtime still
resolved that elected emitter's bootstrap frontier through the staged
per-worker `ef_search` split. That reduced the visible emitter's search budget
even though it was responsible for the full visible output stream.

The scan setup now resolves whether the backend may emit visible tuples before
setting `bootstrap_frontier_limit`:

- visible tuple emitters keep the full serial `effective_ef_search`;
- non-emitting diagnostic contributors keep the existing per-worker split with
  overlap;
- the `ec_hnsw.parallel_ef_overlap` GUC text now scopes the split budget to
  diagnostic contributors.

This is primarily a correctness guard before further performance work. Packet
607's feedback still points to the next performance slice: the elected emitter
drain window closes before contributor rows can be useful.

## Result

The previous default PG18 parallel lane diverged at LIMIT 33 and LIMIT 64. With
the visible emitter on the serial frontier budget, both cases validate against
the serial ordered scan.

LIMIT 33:

```text
Limit (actual time=14.556..15.766 rows=33.00 loops=1)
Bootstrap Expansions: 34
Elements Scored: 34
Heap TIDs Returned: 33
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Poll Limit: No Visible Owner: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

LIMIT 64:

```text
Limit (actual time=15.588..16.593 rows=64.00 loops=1)
Bootstrap Expansions: 65
Elements Scored: 65
Heap TIDs Returned: 64
Parallel Contributor Hidden Publishes: 0
Parallel Contributor Poll Limit: No Visible Owner: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

## Artifacts

- `artifacts/pg18-parallel-limit33-visible-full-ef.log`
- `artifacts/pg18-parallel-limit64-visible-full-ef.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test resolve_bootstrap_frontier_limit --lib`
- `cargo test resolve_parallel_scan_ef_search --lib`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --limit 33 --log-output target/pg18-parallel-limit33-visible-full-ef.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --limit 64 --log-output target/pg18-parallel-limit64-visible-full-ef.log`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18`
- `git diff --check`
