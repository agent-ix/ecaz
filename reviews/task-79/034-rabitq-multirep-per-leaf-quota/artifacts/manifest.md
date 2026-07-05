# Task 79 Packet 034 Manifest: RaBitQ Multi-Representative Per-Leaf Quota

- head SHA: `7c205580ab8a7f6523411ea570d94808a91d6234`
- implementation commit: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/034-rabitq-multirep-per-leaf-quota/`
- timestamp: `2026-06-02T01:51:27-07:00`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- installed clean backend SHA256: `929efde6155ae01ac72dd90395eea24334c1d496fc48b3f738ce7f52a1c1b15a` (installed before packet 033, reused here)
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: `ec_spire.leaf_block_rows=32`
- summary representation: RaBitQ V4 leaf-block summaries with two cluster-mean representatives per block
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- selector: per-leaf block quotas 5/6/7/8, global block cap disabled, radius weight 0.25, no sampled rescue

## Commands

- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/034-rabitq-multirep-per-leaf-quota/suite-rabitq-multirep-per-leaf-quota.json" reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/034-rabitq-multirep-per-leaf-quota/suite-rabitq-multirep-per-leaf-quota.json --manifest-output reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/034-rabitq-multirep-per-leaf-quota/suite-rabitq-multirep-per-leaf-quota.json --log-file reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/suite-manifest.json --log-file reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/suite-manifest.json --results-output reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/report-results.jsonl --log-file reviews/task-79/034-rabitq-multirep-per-leaf-quota/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-multirep-per-leaf-quota.json`: checked-in SuiteConfig for the local per-leaf quota sweep.
- `artifacts/suite-audit.log`: suite audit output; 6 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck. Key lines: `corpus_rows=100000`, `query_rows=1000`.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-multirep.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=16212`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
row	per_leaf_blocks	global_blocks	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
multirep	5	0	0.25	3014842	36.654	42.298	0.8390	2000	fail
multirep	6	0	0.25	3610773	37.284	43.540	0.8720	2000	fail
multirep	7	0	0.25	4196212	38.813	44.785	0.8930	2000	fail
multirep	8	0	0.25	4772824	40.898	47.476	0.9130	2000	fail
```

Interpretation:

- Strict per-leaf quotas do reduce candidates and p50 latency, but they do so by starving recall.
- The best recall row, 8 blocks per leaf, still reaches only 0.9130 recall@10 versus the 0.9925 Task 79 gate.
- This closes per-leaf quota as the likely repair for the candidate-surface problem. The next local path should change RaBitQ block-score discrimination or summary quality, not only redistribute a fixed quota across leaves.
