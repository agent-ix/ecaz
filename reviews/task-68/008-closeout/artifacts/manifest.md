# Task 68 Closeout Manifest

- head SHA: `2c4592b8f9c686ae9b854958674477d7e0d020ac`
- task bucket: `reviews/task-68/008-closeout/`
- packet path: `reviews/task-68/008-closeout/`
- lane: SPIRE build performance closeout
- fixture: local PG18 M5-style Task 68 staging tables
- storage format: `turboquant`
- rerank mode: `rerank_width = 25`
- run surface: isolated one-index-per-table closeout indexes on
  `task68_spire_10k_load_corpus` and `task68_spire_100k_load_corpus`
- timestamp: 2026-05-29 22:45-22:48 America/Los_Angeles

## SuiteConfig

Artifact: `suite.json`

Command:

```text
target/debug/ecaz bench suite audit --config reviews/task-68/008-closeout/artifacts/suite.json
target/debug/ecaz --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/008-closeout/artifacts/suite.json --dry-run --manifest-output reviews/task-68/008-closeout/artifacts/suite-dry-run-manifest.json
cargo run -p ecaz-cli -- --database task68_spire_char --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-68/008-closeout/artifacts/suite.json --manifest-output reviews/task-68/008-closeout/artifacts/suite-manifest.json
```

The final run used `cargo run -p ecaz-cli -- ...` because the installed
`/Users/peter/.cargo/bin/ecaz` binary did not yet include the current
`bench recall --truth-corpus-file` CLI support present in this checkout.

Key result:

```text
[suite:task68-spire-build-closeout] completed=5 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Suite Manifest And Results

Artifacts:

- `suite-manifest.json`
- `results.jsonl`
- `suite-status.log`
- `suite-report.log`
- `results-report.jsonl`

Command:

```text
target/debug/ecaz --log-file reviews/task-68/008-closeout/artifacts/suite-status.log bench suite status --manifest reviews/task-68/008-closeout/artifacts/suite-manifest.json
target/debug/ecaz --log-file reviews/task-68/008-closeout/artifacts/suite-report.log bench suite report --manifest reviews/task-68/008-closeout/artifacts/suite-manifest.json --results-output reviews/task-68/008-closeout/artifacts/results-report.jsonl
```

Key result:

```text
[suite:task68-spire-build-closeout] completed=5 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Host And Fixture Precheck

Artifact: `precheck-host-and-tables.log`

Command: suite step `precheck-host-and-tables`.

Key result:

```text
PostgreSQL 18.3 (Homebrew) on aarch64-apple-darwin25.2.0
corpus_10k_rows: 10000
queries_10k_rows: 200
corpus_100k_rows: 100000
queries_100k_rows: 1000
extname/extversion: ecaz 0.1.1
amname: ec_spire
```

## Build Split And Determinism

Artifacts:

- `build-and-compare-10k.log`
- `build-and-compare-100k.log`

Command: suite steps `build-and-compare-10k` and
`build-and-compare-100k`.

Both steps build two same-seed indexes and compare hierarchy, root routing,
routing centroids, leaf summary, and leaf assignment snapshots.

Key result, 10k:

```text
task68_spire_10k_closeout_idx total_ms=338 heap_scan_ms=138 kmeans_ms=148 assignment_ms=15 draft_leaf_rows_ms=1 top_graph_ms=24
task68_spire_10k_closeout_det_b_idx total_ms=308 heap_scan_ms=112 kmeans_ms=147 assignment_ms=15 draft_leaf_rows_ms=1 top_graph_ms=24
hierarchy/root_routing/routing_centroids/leaf_summary/leaf_assignments: equal
```

Key result, 100k:

```text
task68_spire_100k_closeout_idx total_ms=3418 heap_scan_ms=1307 kmeans_ms=490 assignment_ms=574 draft_leaf_rows_ms=20 top_graph_ms=946
task68_spire_100k_closeout_det_b_idx total_ms=2950 heap_scan_ms=1291 kmeans_ms=485 assignment_ms=573 draft_leaf_rows_ms=16 top_graph_ms=509
hierarchy/root_routing/routing_centroids/leaf_summary/leaf_assignments: equal
```

## Recall Floor

Artifacts:

- `truth-10k-q200-k10.json`
- `truth-100k-q200-k10.json`
- `recall-10k-closeout.log`
- `recall-100k-closeout.log`

Command: suite steps `recall-10k-closeout` and `recall-100k-closeout`.

Both recall steps use `nprobe=16`, because the closeout indexes have
`top_graph_search_list_size=16`.

Key result:

```text
10k: nprobe=16 queries=200 recall@10=0.9995 ndcg@10=1.0000 mean q-time=6.37 ms
100k: nprobe=16 queries=200 recall@10=0.8525 ndcg@10=0.9835 mean q-time=13.78 ms
```

## Prep Probes

Artifacts:

- `snapshot-shape.log`
- `leaf-assignment-snapshot-shape.log`
- `query-counts.log`

These are preparatory schema/query probes from assembling the closeout suite.
They are retained for provenance but are not cited as closeout evidence.
