# Task 120 / 007 Phase 1 Local Containment Artifacts

- head SHA: `e33ac2ae928425231d2ec4907452d613e005ebbd`
- task bucket: `reviews/task-120/007-phase1-local-containment`
- lane: local PG18 SPIRE measurement
- fixture: staged real corpora at `10k`, `50k`, and `100k`
- storage format: `ec_spire`, `bits=4`
- rerank mode: current SPIRE exact/source rerank frontier; no coarse-rerank variant
- isolated one-index-per-table vs shared-table surface: isolated prefixes per scale in database `tqvector_bench_task120`
- timestamp: `2026-06-21T09:31:06-07:00`

The suite used `k=10`, `queries_limit=200`, `iterations=200`, and
`nprobe` sweep `8,16,24,32` for each scale. The local PG18 socket was
`/home/peter/.pgrx`, port `28818`.

Truth-cache files under `artifacts/truth-cache/` are regenerable from the
staged corpora and are intentionally ignored and uncommitted.

## Primary Suite

### `suite.json`

- command/config: checked-in `ecaz bench suite` config for this packet
- config SHA256: `f7bde9a4e5afa423f3bae09f62f9b58351b9272b07784eab44c714d003919919`
- scale/prefixes:
  - `10k`: `task120_phase1_real10k_spire`
  - `50k`: `task120_phase1_real50k_spire`
  - `100k`: `task120_phase1_real100k_spire`

### `suite-manifest.json`, `suite-results.jsonl`, `suite-report.md`, `suite-report-results.jsonl`

- command:

```text
target/debug/ecaz bench suite run --config reviews/task-120/007-phase1-local-containment/artifacts/suite.json --host /home/peter/.pgrx --port 28818 --database tqvector_bench_task120 --resume-from reviews/task-120/007-phase1-local-containment/artifacts/suite-manifest.json --manifest-output reviews/task-120/007-phase1-local-containment/artifacts/suite-manifest.json --results-output reviews/task-120/007-phase1-local-containment/artifacts/suite-results.jsonl --log-file reviews/task-120/007-phase1-local-containment/artifacts/suite-resume-100k-after-space.log
```

- result: completed
- key line from `suite-status.log`: `completed=16 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- steps: precheck, load/recall/latency/storage/pipeline for `10k`, `50k`, and `100k`

### `suite-status.log`

- command:

```text
target/debug/ecaz bench suite status --manifest reviews/task-120/007-phase1-local-containment/artifacts/suite-manifest.json
```

- result: passed
- key line: `completed=16 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

## Key Summaries

### `measurement-summary.txt`

Derived from `suite-results.jsonl`.

Key result lines:

```text
10k	8	0.9920	0.9996	36.16 ms	35.4 ms	46.5 ms	48.5 ms	9.1 MiB	168.0 MiB	9542042	176160768
10k	16	0.9985	0.9999	66.62 ms	64.2 ms	76.5 ms	79.1 ms	9.1 MiB	168.0 MiB	9542042	176160768
10k	24	1.0000	1.0000	96.94 ms	95.7 ms	122.8 ms	130.3 ms	9.1 MiB	168.0 MiB	9542042	176160768
10k	32	1.0000	1.0000	119.10 ms	119.4 ms	134.0 ms	139.0 ms	9.1 MiB	168.0 MiB	9542042	176160768
50k	8	0.8550	0.9885	78.01 ms	75.2 ms	93.2 ms	99.3 ms	42.5 MiB	836.3 MiB	44564480	876924109
50k	16	0.9265	0.9942	140.46 ms	139.9 ms	162.7 ms	179.3 ms	42.5 MiB	836.3 MiB	44564480	876924109
50k	24	0.9480	0.9962	211.96 ms	212.7 ms	260.1 ms	281.0 ms	42.5 MiB	836.3 MiB	44564480	876924109
50k	32	0.9625	0.9975	279.91 ms	291.7 ms	345.9 ms	368.4 ms	42.5 MiB	836.3 MiB	44564480	876924109
100k	8	0.7695	0.9744	109.95 ms	109.2 ms	139.9 ms	150.5 ms	83.6 MiB	1.6 GiB	87660954	1717986918
100k	16	0.8495	0.9835	203.51 ms	208.3 ms	266.2 ms	277.8 ms	83.6 MiB	1.6 GiB	87660954	1717986918
100k	24	0.8940	0.9889	317.05 ms	314.1 ms	378.6 ms	411.5 ms	83.6 MiB	1.6 GiB	87660954	1717986918
100k	32	0.9205	0.9927	423.69 ms	420.6 ms	495.4 ms	543.9 ms	83.6 MiB	1.6 GiB	87660954	1717986918
```

Columns are `scale`, `nprobe`, `recall_at_10`, `ndcg_at_10`,
`recall_mean_q_time`, `latency_mean`, `latency_p95`, `latency_p99`,
`index_size`, `total_size`, `index_bytes`, `total_bytes`.

### `pipeline-stage-containment-summary.txt`

Derived from `pipeline-{scale}-stage-containment.jsonl`.

Key result lines for the local candidate frontier:

```text
10k	8	4	local_candidate_frontier	1984	2000	99.2	16	890.11	0	0.319	routing_miss=16
10k	16	4	local_candidate_frontier	1997	2000	99.85	3	1732.92	0	0.601	routing_miss=3
10k	24	4	local_candidate_frontier	2000	2000	100	0	2520.59	0	0.944
10k	32	4	local_candidate_frontier	2000	2000	100	0	3270.06	0	1.146
50k	8	4	local_candidate_frontier	1710	2000	85.5	290	1808.36	0	0.654	routing_miss=290
50k	16	4	local_candidate_frontier	1853	2000	92.65	147	3606.51	0	1.314	routing_miss=147
50k	24	4	local_candidate_frontier	1896	2000	94.8	104	5365.57	0	1.952	routing_miss=104
50k	32	4	local_candidate_frontier	1925	2000	96.25	75	7138.82	0	2.546	routing_miss=75
100k	8	4	local_candidate_frontier	1539	2000	76.95	461	2484.15	0	0.912	routing_miss=461
100k	16	4	local_candidate_frontier	1699	2000	84.95	301	5076.27	0	1.802	routing_miss=301
100k	24	4	local_candidate_frontier	1788	2000	89.4	212	7620.48	0	2.764	routing_miss=212
100k	32	4	local_candidate_frontier	1841	2000	92.05	159	10237	0	3.659	routing_miss=159
```

The exact/source rerank frontier and final top-k rows match the local candidate
frontier containment for every scale and `nprobe`.

### `pipeline-target-candidate-rank-summary.txt`

Derived from `pipeline-{scale}-target-candidate-rank.jsonl`.

Key result lines:

```text
50k	32	candidate_not_retained	75	0	0	7018.81	7018.81
50k	32	target_candidate_ranked	1925	1925	100	7143.5	7143.5	5.46	10
100k	32	candidate_not_retained	159	0	0	10086.58	10086.58
100k	32	target_candidate_ranked	1841	1841	100	10249.99	10249.99	5.26	10
```

For every retained truth row, `selected_by_prefix=100%`; p95 approximate rank
for retained rows is at most `10`. Misses are absent from the candidate
frontier rather than lost by exact/source rerank.

### `pipeline-funnel-summary.txt`

Derived from `pipeline-{scale}-funnel.jsonl`.

Key result lines:

```text
50k	32	200	6400	1427764	7138.82	1147504096	5737520	315.693	368.2	64.523	76.512	332.54	7069.5
100k	32	200	6400	2047399	10237	1645204612	8226023	452.201	522.994	90.603	118.107	851.4	10193.76
```

Columns are `scale`, `nprobe`, `queries`, `route_sum`, `candidate_sum`,
`avg_candidates`, `object_bytes_sum`, `avg_object_bytes`,
`leaf_object_read_ms`, `row_score_ms`, `materialize_ms`, `heap_append_ms`,
`avg_unique_heap_blocks`, `avg_heap_block_transitions`.

### `pipeline-target-block-rank-summary.txt`

Derived from `pipeline-{scale}-target-block-rank.jsonl`.

Key result line pattern:

```text
100k	32	not_found_in_routed_leaves	2000	0	0
```

All scale/`nprobe` slices report `not_found_in_routed_leaves` for all 2,000
truth rows in the target-block rank snapshot. That block-rank snapshot is not
decision-grade for local route/block attribution in this packet; use the target
candidate-rank and final containment rows for the Phase 1 local conclusion.

## Raw Measurement Logs

### Load logs

- files: `load-10k-spire.log`, `load-50k-spire.log`, `load-100k-spire.log`
- command source: `suite.json`
- result: all load steps succeeded

### Recall logs

- files: `recall-10k-spire.log`, `recall-50k-spire.log`, `recall-100k-spire.log`
- command source: `suite.json`
- result: all recall steps succeeded
- key values: summarized in `measurement-summary.txt`

### Latency logs

- files: `latency-10k-spire.log`, `latency-50k-spire.log`, `latency-100k-spire.log`
- command source: `suite.json`
- result: all latency steps succeeded
- key values: summarized in `measurement-summary.txt`

### Storage logs

- files: `storage-10k-spire.log`, `storage-50k-spire.log`, `storage-100k-spire.log`
- command source: `suite.json`
- result: all storage steps succeeded
- key values: summarized in `measurement-summary.txt`

### SPIRE pipeline logs and JSONL

- files:
  - `pipeline-10k-spire.log`
  - `pipeline-10k-funnel.jsonl`
  - `pipeline-10k-stage-containment.jsonl`
  - `pipeline-10k-target-block-rank.jsonl`
  - `pipeline-10k-target-candidate-rank.jsonl`
  - `pipeline-50k-spire.log`
  - `pipeline-50k-funnel.jsonl`
  - `pipeline-50k-stage-containment.jsonl`
  - `pipeline-50k-target-block-rank.jsonl`
  - `pipeline-50k-target-candidate-rank.jsonl`
  - `pipeline-100k-spire.log`
  - `pipeline-100k-funnel.jsonl`
  - `pipeline-100k-stage-containment.jsonl`
  - `pipeline-100k-target-block-rank.jsonl`
  - `pipeline-100k-target-candidate-rank.jsonl`
- command source: `suite.json`
- result: all pipeline steps succeeded
- line counts:
  - funnel JSONL: 800 rows per scale
  - stage-containment JSONL: 4,800 rows per scale
  - target-block-rank JSONL: 8,000 rows per scale
  - target-candidate-rank JSONL: 8,000 rows per scale

## Setup and Provenance Logs

- `cargo-build-ecaz-cli.log`: local CLI build provenance
- `install-ecaz-pg18.log`: installed the current PG18 extension before measurement
- `create-task120-database.log`: created `tqvector_bench_task120`
- `create-extension-task120.log`: installed `ecaz` in `tqvector_bench_task120`; `ecaz_build_profile()` returned `release`
- `precheck-host.log`: PG18 host settings captured by the suite
- `suite-audit.log`, `suite-dry-run.log`, `suite-dry-run-manifest.json`: suite audit/dry-run provenance
- `db-task120-exists.log`, `pg18-database-sizes.log`, `pg18-database-sizes-after-old-db-drop.log`, `db-size-after-disk-full.log`, `db-size-after-drop-generated-surfaces.log`: local storage provenance while recovering from a disk-full retry
- `suite-run.log`, `suite-resume-100k.log`, `suite-resume-100k-after-space.log`, `suite-manifest-disk-full.json`: failed/resumed run provenance; the cited result source is the final `suite-manifest.json`
- `drop-task120-generated-surfaces.log`, `drop-task120-partial-100k-after-old-db-drop.log`, `drop-old-task111h-corrected-100k-v9.log`: cleanup logs used to free space and restart only the partial 100k slice
