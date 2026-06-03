# Task 79 Packet 031 Manifest: RaBitQ Multi-Representative Sampled Rescue

- head SHA: `808b7e072451af69069e95cbecffbdb58c8260d2`
- implementation commit: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/031-rabitq-multirep-sampled-rescue/`
- timestamp: `2026-06-02T07:50:12Z`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: clustered `ec_spire.leaf_block_rows=32`
- summary representation: RaBitQ V4 leaf-block summaries with two representatives per block
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- sampled selector: final global block cap 640, radius weight 0.25, probe windows 896/1024, sample rows 1/2, summary prior weight 0.8

## Commands

- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/031-rabitq-multirep-sampled-rescue/suite-rabitq-multirep-sampled-rescue.json" reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/031-rabitq-multirep-sampled-rescue/suite-rabitq-multirep-sampled-rescue.json --manifest-output reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/031-rabitq-multirep-sampled-rescue/suite-rabitq-multirep-sampled-rescue.json --log-file reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/suite-manifest.json --log-file reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/suite-manifest.json --results-output reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/report-results.jsonl --log-file reviews/task-79/031-rabitq-multirep-sampled-rescue/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-multirep-sampled-rescue.json`: checked-in SuiteConfig for the sampled-rescue sweep.
- `artifacts/suite-audit.log`: suite audit output; 6 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-multirep.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=16399`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
row	final_blocks	probe_blocks	sample_rows	prior_weight	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
summary	640	0	0	0.8	0.25	4050758	45.272	53.010	0.9870	2000	fail
sampled	640	896	1	0.8	0.25	4229958	49.407	57.603	0.9870	1933	fail
sampled	640	1024	1	0.8	0.25	4255558	49.596	57.262	0.9870	1933	fail
sampled	640	896	2	0.8	0.25	4409158	50.826	60.268	0.9870	1878	fail
```

Interpretation:

- Sampled rescue does not recover recall at cap640. All sampled rows stay at recall@10 0.9870.
- Sampling adds candidate surface and latency: sample1 rows rise to about 4.23M-4.26M candidates and about 49.5 ms p50; sample2 rises to 4.41M candidates and 50.826 ms p50.
- Sampling causes under-return: 1933 returned rows for sample1 and 1878 for sample2, versus 2000 for summary-only.
- This closes the existing sampled selector as the next Task 79 fix for the multi-representative RaBitQ path.
