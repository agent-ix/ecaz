# Task 79 Packet 030 Manifest: RaBitQ Multi-Representative Rank Diagnostic

- head SHA: `808b7e072451af69069e95cbecffbdb58c8260d2`
- implementation commit: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/030-rabitq-multirep-rank-diagnostic/`
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
- diagnostic row: global leaf-block cap 768, radius weight 0.25

## Commands

- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/030-rabitq-multirep-rank-diagnostic/suite-rabitq-multirep-rank-diagnostic.json" reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/030-rabitq-multirep-rank-diagnostic/suite-rabitq-multirep-rank-diagnostic.json --manifest-output reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/030-rabitq-multirep-rank-diagnostic/suite-rabitq-multirep-rank-diagnostic.json --log-file reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/suite-manifest.json --log-file reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/suite-manifest.json --results-output reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/report-results.jsonl --log-file reviews/task-79/030-rabitq-multirep-rank-diagnostic/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-multirep-rank-diagnostic.json`: checked-in SuiteConfig for the local rank diagnostic.
- `artifacts/register-leaf-block-rank-function.sql`: packet-local SQL registration for the rank diagnostic function.
- `artifacts/suite-audit.log`: suite audit output; 4 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck.
- `artifacts/register-leaf-block-rank-function.log`: SQL function registration log.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-multirep.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=16278`.
- `artifacts/pipeline-leaf-block-rank-100k-rabitq-block32-multirep-global768-rw025.log`: pipeline log with candidate, latency, and recall metrics.
- `artifacts/leaf-block-rank-100k-rabitq-block32-multirep-global768-rw025.jsonl`: 2,000 exact-target block-rank rows.
- `artifacts/leaf-block-rank-analysis.md`: packet-local analysis of the JSONL rank file.

## Key Results

Pipeline row:

| candidates | p50 | p95 | recall@10 |
| ---: | ---: | ---: | ---: |
| 4,860,415 | 47.616 ms | 57.164 ms | 0.9905 |

Block-rank diagnostic:

| status | count |
| --- | ---: |
| `block_ranked` | 1,995 |
| `not_found_in_routed_leaves` | 5 |

| cap | selected exact top-10 targets | missed |
| ---: | ---: | ---: |
| 512 | 1,959 | 41 |
| 640 | 1,974 | 26 |
| 768 | 1,981 | 19 |
| 896 | 1,987 | 13 |
| 1024 | 1,989 | 11 |
| 1280 | 1,994 | 6 |

Rank distribution for routed/ranked exact top-10 targets:

| p50 | p90 | p95 | p97.5 | p99 | max |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 7 | 128 | 245 | 401 | 651 | 1309 |

Interpretation:

- The cap768 row is short by 4 exact top-10 targets versus the recall gate, but it misses the p50 gate.
- The cap640 row is the latency target, but it is short by 11 exact top-10 targets versus the recall gate.
- The rank file reaches the recall gate at cap896, which would exceed the 5.2M candidate gate for block32.
- The next local slice should recover near-cap ranked targets without increasing the final block cap, or improve score calibration enough to reorder the same 640 selected blocks.
