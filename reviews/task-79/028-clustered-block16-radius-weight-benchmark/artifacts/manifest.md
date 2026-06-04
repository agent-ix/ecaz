# Task 79 Packet 028 Manifest: Clustered Block16 Radius-Weight Benchmark

- head SHA: `e2eaed0d5ad9559346cf9c9bf0ecd494a1e2c6e1`
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/028-clustered-block16-radius-weight-benchmark/`
- timestamp: `2026-06-02T06:41:22Z`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: clustered `ec_spire.leaf_block_rows=16`
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- candidate pruning: global leaf-block cap sweep, `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`, `global_probe_blocks=0`, `sample_rows_per_block=0`, summary prior weight 0.8, radius weight sweep 0.0/0.25

## Commands

- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/028-clustered-block16-radius-weight-benchmark/suite-rabitq-clustered-block16-radius-weight.json" reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/028-clustered-block16-radius-weight-benchmark/suite-rabitq-clustered-block16-radius-weight.json --manifest-output reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/028-clustered-block16-radius-weight-benchmark/suite-rabitq-clustered-block16-radius-weight.json --log-file reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/028-clustered-block16-radius-weight-benchmark/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-clustered-block16-radius-weight.json`: checked-in SuiteConfig for the local RaBitQ clustered block16 radius-weight sweep.
- `artifacts/suite-audit.log`: suite audit output; 10 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=10 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block16-clustered.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing: ... total_ms=17682`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
step	global_blocks	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
pipeline-100k-rabitq-n128-f8-b0-tg96-block16-global0-rw0	0	0	15506227	63.920	78.431	0.9975	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block16-global1024-rw0	1024	0	3250670	45.323	54.895	0.9740	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block16-global1024-rw025	1024	0.25	3263288	43.323	54.697	0.9805	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block16-global1280-rw0	1280	0	4063239	45.760	52.058	0.9805	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block16-global1280-rw025	1280	0.25	4078731	46.912	53.758	0.9860	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block16-global1536-rw0	1536	0	4875863	47.958	60.372	0.9850	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block16-global1536-rw025	1536	0.25	4894281	48.613	55.067	0.9890	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block16-global1664-rw025	1664	0.25	5301755	50.025	58.927	0.9900	2000	fail
```

Interpretation:

- The best under-cap block16 row is `global1536/radius0.25`: 4,894,281 candidates, p50 48.613 ms, p95 55.067 ms, recall@10 0.9890.
- The only row that reaches 0.9900 recall is `global1664/radius0.25`, but it scans 5,301,755 candidates and still misses the 0.9925 recall gate.
- Compared with the unbounded block16 baseline, `global1536/radius0.25` cuts candidates from 15,506,227 to 4,894,281 and p50 from 63.920 ms to 48.613 ms, but recall drops by 0.85 percentage points.
- Compared with packet 027's best clustered block32 row, block16 improves the candidate count but worsens recall enough that it is not the next acceptable route.
- This negative result reinforces the packet 026 diagnostic: single summary plus radius weighting does not discriminate enough of the block-rank tail. The next direct fix should enrich the per-block summary representation rather than continuing cap-only or radius-only tuning.
