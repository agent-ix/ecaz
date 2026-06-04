# Task 79 Packet 039 Manifest: RaBitQ K3 Routing-Breadth Sweep

- head SHA: `cef415310813e80ccbcaf41fe5a8b8c83b536dda`
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/039-rabitq-k3-routing-breadth-sweep/`
- timestamp: `2026-06-02T04:26:19-07:00`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- code baseline: packet 038 fast-path code, `46d83192e` (`Optimize SPIRE leaf block summary scoring`)
- installed backend SHA256: `210566e905947116d8d9aa6eb718d99368302aa02aca5e17edbc71da96e41a10`
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active index `task79_surface_100k_idx`
- index provenance: reused the local k=3 RaBitQ index rebuilt in packet 037, `reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-k3-two-stage.log`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: `ec_spire.leaf_block_rows=32`
- summary representation: existing RaBitQ V4 leaf-block summaries with three cluster-mean representatives per block from packet 037
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: adaptive nprobe off; `nprobe` sweep 64/72/80/88/96
- selector: full k3 summary scoring, global block cap 736, radius weight 0.25, no sampled rescue

## Commands

- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/039-rabitq-k3-routing-breadth-sweep/suite-rabitq-k3-routing-breadth-sweep.json" reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/039-rabitq-k3-routing-breadth-sweep/suite-rabitq-k3-routing-breadth-sweep.json --manifest-output reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/039-rabitq-k3-routing-breadth-sweep/suite-rabitq-k3-routing-breadth-sweep.json --log-file reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-manifest.json --log-file reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-manifest.json --results-output reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/report-results.jsonl --log-file reviews/task-79/039-rabitq-k3-routing-breadth-sweep/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-k3-routing-breadth-sweep.json`: checked-in SuiteConfig for the local routing-breadth sweep.
- `artifacts/suite-audit.log`: suite audit output; 2 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=2 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact route/candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-k3-index.log`: corpus/query/index/GUC precheck. Key lines: `corpus_rows=100000`, `query_rows=1000`, `task79_surface_100k_idx`.
- `artifacts/pipeline-100k-rabitq-k3-fast-global736-routing-sweep-rw025.log`: pipeline log with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-100k-rabitq-k3-fast-global736-routing-sweep-rw025.jsonl`: per-nprobe funnel output.

## Key Results

The compact result table is:

```text
nprobe	route_sum	selected_pid_sum	candidates	object_bytes_sum	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
64	12800	12800	4657560	9285184652	38.277	44.475	0.9815	2000	fail_recall
72	14400	14400	4657439	10414027244	41.611	46.625	0.9865	2000	fail_recall
80	16000	16000	4657540	11542973836	43.430	51.223	0.9895	2000	fail_recall
88	17600	17600	4657349	12689352924	45.399	54.408	0.9910	2000	fail_recall_p50
96	19200	19200	4657668	13816992816	47.936	54.885	0.9925	2000	fail_p50
```

Interpretation:

- Routing breadth is a strong latency lever: p50 falls from 47.936 ms at nprobe96 to 38.277 ms at nprobe64.
- Recall is the limiter: every row below nprobe96 misses the 0.9925 recall point observed at the current best recipe.
- Candidate rows stay roughly flat near 4.657M because the global736 block cap still fills from the routed leaves; lower nprobe reduces route/object-read work, not the selected row surface.
- This closes simple nprobe reduction on the current k=3/global736 index shape as a strict Task 79 fix.
