# Task 120 / 008 Phase 2 RaBitQ Block-Pruning Artifacts

- head SHA: `f919874bfa6591c7b6722b7cc27b5aafa64cf0ef`
- task bucket: `reviews/task-120/008-phase2-rabitq-block-pruning`
- lane: local PG18 SPIRE measurement
- fixture: staged real corpora at `10k`, `50k`, and `100k`
- storage format: `ec_spire`, `storage_format=rabitq`, `bits=4`
- rerank mode: current SPIRE exact/source rerank with `rerank_width=25`
- isolated one-index-per-table vs shared-table surface: isolated prefixes per scale in database `tqvector_bench_task120`
- timestamp: `2026-06-21T10:49:17-07:00`

This packet uses a bespoke `ecaz bench suite` config instead of the canonical
current lane config because Task 120 Phase 2 needs SPIRE-only recursive RaBitQ
surfaces plus the `spire-pipeline` selected-leaf-block diagnostics. The standard
lane configs do not carry that diagnostic matrix.

The corrected measurement uses recursive SPIRE (`recursive_fanout=8`) because
the earlier flat SPIRE diagnostic showed `leaf_block_available_count=0`; leaf
block summaries are exercised on the recursive path. The final A/B evidence in
this packet is therefore the `full` vs `l2` recursive RaBitQ matrix only. The
aborted flat `g128` artifacts are not cited.

The suite used `k=10`, `queries_limit=200`, `iterations=200`, `concurrency=1`,
and `nprobe` sweep `8,16,24,32` for each scale. The local PG18 socket was
`/home/peter/.pgrx`, port `28818`.

Truth-cache files under `artifacts/truth-cache/` are regenerable from the staged
corpora and are intentionally ignored and uncommitted.

## Primary Suite

### `suite.json`

- command/config: checked-in `ecaz bench suite` config for this packet
- config SHA256: `1ede2a2a65e80944a3bca9bc095f154fc70996f8ddac8b82aa08556fb0c49745`
- index reloptions:
  - `nlists=128`
  - `recursive_fanout=8`
  - `nprobe=24`
  - `rerank_width=25`
  - `boundary_replica_count=0`
  - `top_graph_enabled=1`
  - `top_graph_degree=32`
  - `top_graph_build_list_size=100`
  - `top_graph_search_list_size=96`
  - `storage_format=rabitq`
- load/build `PGOPTIONS`: `ec_spire.leaf_block_rows=64`, `ec_spire.leaf_block_summary_representatives=2`
- scale/prefixes:
  - `10k`: `task120_phase2_real10k_spire_rabitq_f8_b64_l2`
  - `50k`: `task120_phase2_real50k_spire_rabitq_f8_b64_l2`
  - `100k`: `task120_phase2_real100k_spire_rabitq_f8_b64_l2`

### `suite-manifest.json`, `suite-results.jsonl`

- command:

```text
target/debug/ecaz bench suite run --config reviews/task-120/008-phase2-rabitq-block-pruning/artifacts/suite.json --host /home/peter/.pgrx --port 28818 --database tqvector_bench_task120 --manifest-output reviews/task-120/008-phase2-rabitq-block-pruning/artifacts/suite-manifest.json --results-output reviews/task-120/008-phase2-rabitq-block-pruning/artifacts/suite-results.jsonl --log-file reviews/task-120/008-phase2-rabitq-block-pruning/artifacts/suite-run.log
```

- result: completed
- key line from `suite-status.log`: `completed=25 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- steps: precheck, load/storage, and `full`/`l2` recall, latency, and pipeline runs for `10k`, `50k`, and `100k`

### `suite-status.log`

- command:

```text
target/debug/ecaz bench suite status --manifest reviews/task-120/008-phase2-rabitq-block-pruning/artifacts/suite-manifest.json
```

- result: passed
- key line: `completed=25 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`

## Variant Definitions

- `full`: disables all leaf block pruning caps with session GUCs:
  `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`,
  `ec_spire.leaf_block_pruning_max_global_blocks=0`,
  `ec_spire.leaf_block_pruning_global_probe_blocks=0`,
  `ec_spire.leaf_block_pruning_sample_rows_per_block=0`.
- `l2`: keeps the same index and query matrix, but sets
  `ec_spire.leaf_block_pruning_max_blocks_per_leaf=2`,
  leaves global caps disabled, and uses
  `ec_spire.leaf_block_pruning_summary_radius_weight=1.0`,
  `ec_spire.leaf_block_pruning_route_prior_weight=0.0`.

## Key Summaries

### `recall-latency-summary.txt`

Derived from `suite-results.jsonl`.

Key result lines:

```text
10k	full	32	0.9965	0.9928	0.9983	0.9999	8.49	9.51	10.3
10k	l2	32	0.9855	0.9793	0.9899	0.9991	8.12	8.99	10.1
50k	full	32	0.9725	0.9644	0.9788	0.9987	16.0	18.4	19.2
50k	l2	32	0.5505	0.5286	0.5722	0.9524	11.8	13.0	14.9
100k	full	32	0.9310	0.9190	0.9413	0.9934	26.8	30.4	31.9
100k	l2	32	0.5060	0.4841	0.5279	0.9323	14.9	16.3	16.9
```

Columns are `scale`, `variant`, `nprobe`, `recall_at_k`,
`recall_ci95_low`, `recall_ci95_high`, `ndcg_at_k`, `latency_mean_ms`,
`latency_p95_ms`, `latency_p99_ms`.

### `block-pruning-summary.txt`

Derived from `pipeline-{scale}-rabitq-{full,l2}-stage-containment.jsonl` rows
where `stage_name=selected_leaf_blocks`.

Key result lines:

```text
50k	full	32	39513	39513	0	2326779	63143738	1897002720
50k	l2	32	39513	12800	26713	784248	63143738	1897002720
100k	full	32	83532	83532	0	5165224	133249592	4210922560
100k	l2	32	83532	12800	70732	806794	133249592	4210922560
```

Columns are `scale`, `variant`, `nprobe`, `available_blocks`,
`selected_blocks`, `skipped_blocks`, `candidates`, `summary_bytes`, `row_bytes`.

### `block-pruning-comparison.txt`

Derived from `block-pruning-summary.txt`.

Key result lines:

```text
50k	32	2326779	784248	66.29	39513	12800	67.61	26713
100k	32	5165224	806794	84.38	83532	12800	84.68	70732
```

Columns are `scale`, `nprobe`, `full_candidates`, `l2_candidates`,
`candidate_reduction_pct`, `full_selected_blocks`, `l2_selected_blocks`,
`selected_block_reduction_pct`, `l2_skipped_blocks`.

### `storage-summary.txt`

Derived from `suite-results.jsonl`.

Key result lines:

```text
10k	10000	158.8 MiB	10.0 MiB	168.8 MiB	17695.5 B	9.7 MiB	1017.4 B
50k	50000	793.8 MiB	43.2 MiB	837.0 MiB	17553.0 B	42.1 MiB	882.9 B
100k	100000	1.6 GiB	84.7 MiB	1.6 GiB	17534.0 B	82.5 MiB	865.0 B
```

Columns are `scale`, `rows`, `table_bytes_pretty`, `all_indexes_pretty`,
`total_pretty`, `per_row_total`, `spire_index_size`, `spire_index_per_row`.

## Result Interpretation

`l2` proves that recursive leaf block summaries are active and can prune work,
but it is not recall-safe at 50k/100k. At `nprobe=32`, `l2` reduces candidates
by `66.29%` at 50k and `84.38%` at 100k, but recall falls from `0.9725` to
`0.5505` at 50k and from `0.9310` to `0.5060` at 100k. The latency win is
therefore purchased by dropping too many true-neighbor rows.

This packet does not use target-block-rank output for route-vs-leaf-vs-block
truth attribution. The Phase 1 reviewer correctly flagged that upstream
target-block-rank attribution as non-decision-grade. The Phase 2 conclusion here
rests on A/B final recall, latency, storage, and selected-leaf-block candidate
counts.

## Raw Measurement Logs

### Load and storage logs

- files: `load-{10k,50k,100k}-rabitq-b64.log`
- files: `storage-{10k,50k,100k}-rabitq-b64.log`
- command source: `suite.json`
- result: all load and storage steps succeeded

### Recall and latency logs

- files: `recall-{10k,50k,100k}-rabitq-{full,l2}.log`
- files: `latency-{10k,50k,100k}-rabitq-{full,l2}.log`
- command source: `suite.json`
- result: all recall and latency steps succeeded
- key values: summarized in `recall-latency-summary.txt`

### SPIRE pipeline logs and JSONL

- files:
  - `pipeline-{10k,50k,100k}-rabitq-{full,l2}.log`
  - `pipeline-{10k,50k,100k}-rabitq-{full,l2}-funnel.jsonl`
  - `pipeline-{10k,50k,100k}-rabitq-{full,l2}-stage-containment.jsonl`
  - `pipeline-{10k,50k,100k}-rabitq-{full,l2}-leaf-block-rank.jsonl`
  - `pipeline-{10k,50k,100k}-rabitq-{full,l2}-target-block-rank.jsonl`
  - `pipeline-{10k,50k,100k}-rabitq-{full,l2}-target-candidate-rank.jsonl`
- command source: `suite.json`
- result: all pipeline steps succeeded
- cited values: selected-leaf-block pruning rows summarized in
  `block-pruning-summary.txt` and `block-pruning-comparison.txt`

## Setup and Provenance Logs

- `precheck-host.log`: PG18 host settings captured by the suite, including
  `ec_spire.leaf_block_rows=64` and `ec_spire.leaf_block_summary_representatives=2`
- `suite-audit.log`, `suite-dry-run.log`, `suite-dry-run-manifest.json`: suite
  audit/dry-run provenance
- `suite-run.log`, `suite-manifest.json`, `suite-results.jsonl`,
  `suite-status.log`: final suite run and structured results
- `diagnostic-create-index-recursive-set-b64.log`,
  `diagnostic-recursive-perleaf-cap2-10k-n32-q20.log`,
  `diagnostic-recursive-perleaf-cap-sweep-10k-n32-q20.log`: small pre-suite
  diagnostics used to confirm recursive leaf-block summaries and per-leaf caps
  were active before the final matrix
