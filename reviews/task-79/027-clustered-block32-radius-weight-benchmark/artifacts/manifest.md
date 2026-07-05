# Task 79 Packet 027 Manifest: Clustered Block32 Radius-Weight Benchmark

- head SHA: `0b9c8f0642217a7c47ce3470f2c442ea9f8c1bba`
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/027-clustered-block32-radius-weight-benchmark/`
- timestamp: `2026-06-01T23:16:49-07:00`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: clustered `ec_spire.leaf_block_rows=32`
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- candidate pruning: global leaf-block cap sweep, `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`, `global_probe_blocks=0`, `sample_rows_per_block=0`, summary prior weight 0.8, radius weight sweep 0.0/0.25/0.5

## Commands

- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/027-clustered-block32-radius-weight-benchmark/suite-rabitq-clustered-block32-radius-weight.json" reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/027-clustered-block32-radius-weight-benchmark/suite-rabitq-clustered-block32-radius-weight.json --manifest-output reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/027-clustered-block32-radius-weight-benchmark/suite-rabitq-clustered-block32-radius-weight.json --log-file reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/027-clustered-block32-radius-weight-benchmark/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-clustered-block32-radius-weight.json`: checked-in SuiteConfig for the local RaBitQ clustered block32 radius-weight sweep.
- `artifacts/suite-audit.log`: suite audit output; 13 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=13 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-clustered.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing: ... total_ms=15423`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
label	global_block_cap	radius_weight	candidate_sum	latency_p50_ms	latency_p95_ms	recall_at_10	pass_recall
block32-global0-rw0	0	0.00	15506227	63.228	74.196	0.9975	yes
block32-global640-rw0	640	0.00	4022736	43.257	49.947	0.9765	no
block32-global640-rw025	640	0.25	4057183	42.936	50.079	0.9820	no
block32-global768-rw0	768	0.00	4826773	45.382	54.468	0.9825	no
block32-global768-rw025	768	0.25	4865680	47.061	56.518	0.9865	no
block32-global768-rw05	768	0.50	4881399	45.388	55.231	0.9810	no
block32-global832-rw0	832	0.00	5228826	46.131	52.020	0.9850	no
block32-global832-rw025	832	0.25	5269892	48.496	56.537	0.9885	no
block32-global896-rw0	896	0.00	5631072	47.344	55.108	0.9860	no
block32-global896-rw025	896	0.25	5674919	47.620	54.526	0.9900	yes
block32-global1024-rw025	1024	0.25	6483892	50.705	57.513	0.9915	yes
```

Interpretation:

- The first recall-passing clustered block32 setting is `global896/radius0.25`: 5,674,919 candidates, p50 47.620 ms, p95 54.526 ms, recall@10 0.9900.
- The higher-margin `global1024/radius0.25` setting improves recall only to 0.9915 while raising candidates to 6,483,892 and p50 to 50.705 ms.
- Compared with this packet's unbounded block32 baseline, `global896/radius0.25` cuts candidates from 15,506,227 to 5,674,919 and p50 from 63.228 ms to 47.620 ms.
- Compared with packet 025's prior recall-passing clustered block64 `global768/rw0` point, this cuts candidates from 9,525,502 to 5,674,919 and p50 from 56.486 ms to 47.620 ms.
- The result is an improvement, but it does not eliminate the core issue: the current single-summary/radius block score still requires millions of candidates to hit recall. This reinforces the packet 026 diagnostic conclusion that the next structural step should add richer per-block representative information instead of another cap-only tuning pass.
