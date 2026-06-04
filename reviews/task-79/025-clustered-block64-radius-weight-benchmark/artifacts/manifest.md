# Task 79 Packet 025 Artifact Manifest

- Packet: `reviews/task-79/025-clustered-block64-radius-weight-benchmark`
- Task: `plan/tasks/79-spire-candidate-surface-reduction.md`
- Head SHA: `d5c15c6c314e17646c38aa9e9eac99617fc0fbfb`
- Timestamp: `2026-06-01T22:10:29-07:00`
- Lane: local PG18, Intel local
- Fixture: existing `task79_spire_candidate_surface` database, `task79_surface_100k_corpus`, `task79_surface_100k_queries`
- Surface: shared Task 79 100k fixture table with one rebuilt index, `task79_surface_100k_idx`
- Storage format: RaBitQ
- Index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with search list size 96
- Block layout: `ec_spire.leaf_block_rows=64`, clustered by packet 024 code before V3 summary creation
- Query shape: 200 queries, `nprobe=96`, `rerank_width=25`, recall@10, local store overlap, cost snapshot, production read profile
- Truth corpus: `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- Suite config: `../suite-rabitq-clustered-block64-radius-weight.json`
- Suite config SHA256: `a42f25c189eaa4269e5504b7aba431b61724aaee2d8527b651f33a0258269081`
- Installed backend SHA256: `5e9aec3491f7c07cde2be38f1476f65225b94207270afbfc98cbb78947e99f8d`

## Commands

- `script -q -c "cargo build -p ecaz-cli" reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/025-clustered-block64-radius-weight-benchmark/suite-rabitq-clustered-block64-radius-weight.json" reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/suite-audit.log`
- `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/install-ecaz-pg18.log`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/pg18-restart.log restart -m fast`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/025-clustered-block64-radius-weight-benchmark/suite-rabitq-clustered-block64-radius-weight.json --manifest-output reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/suite-dry-run.log`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/025-clustered-block64-radius-weight-benchmark/suite-rabitq-clustered-block64-radius-weight.json --log-file reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/suite-run.log`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/suite-status.log`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/025-clustered-block64-radius-weight-benchmark/artifacts/suite-report.log`

No AWS commands were used for this packet.

## Setup Artifacts

- `cargo-build-ecaz-cli.log`: local CLI build log.
- `suite-audit.log`: suite audit; passed 15 steps.
- `install-ecaz-pg18.log`: local PG18 extension install log; installed backend sha256 `5e9aec3491f7c07cde2be38f1476f65225b94207270afbfc98cbb78947e99f8d`.
- `pg18-restart.log`: local PG18 restart log.
- `suite-dry-run.log`, `suite-dry-run-manifest.json`: suite dry-run evidence.
- `suite-run.log`, `suite-manifest.json`, `results.jsonl`: canonical suite run evidence; 15 completed, 0 failed, 0 skipped.
- `suite-status.log`: post-run status; completed=15 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0.
- `suite-report.log`, `report-results.jsonl`: generated report and parsed results.
- `compact-results.tsv`: compact table of candidate, latency, recall, and gate result.
- `precheck-existing-task79-surface.log`: fixture and GUC precheck; corpus rows=100000, query rows=1000, new radius-weight GUC present.
- `rebuild-100k-rabitq-n128-f8-b0-tg96-block64-clustered.log`: clustered block64 rebuild log; `ec_spire_ambuild_timing ... draft_leaf_inputs_ms=4438 ... total_ms=14152`.

Prior unclustered block64 rebuild context from earlier packets is not a controlled same-packet delta: packet 015 had `total_ms=9616`, packet 023 had `total_ms=13216`, packet 021 had `total_ms=14983`. Packet 025's controlled build statement is the local clustered rebuild total: `total_ms=14152`.

## Pipeline Artifacts

Each row has a human-readable pipeline log and a funnel JSONL artifact.

| Row | Pipeline log | Funnel |
| --- | --- | --- |
| global0 rw0 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global0-rw0.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global0-rw0.jsonl` |
| global384 rw0 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global384-rw0.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global384-rw0.jsonl` |
| global384 rw0.25 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global384-rw025.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global384-rw025.jsonl` |
| global384 rw0.5 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global384-rw05.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global384-rw05.jsonl` |
| global384 rw1.0 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global384-rw1.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global384-rw1.jsonl` |
| global400 rw0 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global400-rw0.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global400-rw0.jsonl` |
| global400 rw0.25 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global400-rw025.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global400-rw025.jsonl` |
| global400 rw0.5 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global400-rw05.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global400-rw05.jsonl` |
| global416 rw0 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global416-rw0.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global416-rw0.jsonl` |
| global416 rw0.25 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global416-rw025.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global416-rw025.jsonl` |
| global512 rw0 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global512-rw0.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global512-rw0.jsonl` |
| global512 rw0.25 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global512-rw025.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global512-rw025.jsonl` |
| global768 rw0 | `pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global768-rw0.log` | `funnel-100k-rabitq-n128-f8-b0-tg96-block64-global768-rw0.jsonl` |

## Gate Criteria

- Baseline reference in this packet: global0, 15,506,227 candidates, p50 63.515 ms, recall@10 0.9975.
- Recall gate: recall@10 >= 0.9925.
- Candidate gate: candidate_sum <= 5,200,000 over 200 queries.
- Latency target: p50 <=45 ms or >=25% improvement, but latency is invalid without candidate and recall gates.
- Retained/returned surface: all rows retain 5,000 candidates for heap rerank and return 2,000 rows.

## Key Results

| global_cap | radius_weight | candidate_sum | p50 | p95 | recall@10 | gate |
| ---: | ---: | ---: | ---: | ---: | ---: | --- |
| 0 | 0 | 15,506,227 | 63.515 ms | 74.369 ms | 0.9975 | FAIL: candidate baseline |
| 384 | 0 | 4,764,181 | 44.606 ms | 52.119 ms | 0.9690 | FAIL: recall |
| 384 | 0.25 | 4,798,986 | 44.745 ms | 53.519 ms | 0.9760 | FAIL: recall |
| 384 | 0.5 | 4,814,533 | 44.590 ms | 52.203 ms | 0.9725 | FAIL: recall |
| 384 | 1.0 | 4,831,078 | 45.493 ms | 51.858 ms | 0.9205 | FAIL: recall |
| 400 | 0 | 4,962,846 | 45.598 ms | 57.094 ms | 0.9710 | FAIL: recall |
| 400 | 0.25 | 4,998,286 | 44.943 ms | 55.282 ms | 0.9810 | FAIL: recall |
| 400 | 0.5 | 5,014,237 | 46.148 ms | 53.226 ms | 0.9755 | FAIL: recall |
| 416 | 0 | 5,160,801 | 45.491 ms | 51.783 ms | 0.9720 | FAIL: recall |
| 416 | 0.25 | 5,197,973 | 45.796 ms | 57.740 ms | 0.9835 | FAIL: recall |
| 512 | 0 | 6,351,155 | 49.316 ms | 56.465 ms | 0.9825 | FAIL: candidates and recall |
| 512 | 0.25 | 6,392,555 | 48.363 ms | 55.089 ms | 0.9870 | FAIL: candidates and recall |
| 768 | 0 | 9,525,502 | 56.486 ms | 68.121 ms | 0.9930 | FAIL: candidates and latency |

## Outcome

Clustered block64 summaries do not satisfy Task 79. The best candidate-budget row was `global416/rw0.25` with 5,197,973 candidates, p50 45.796 ms, and recall@10 0.9835, which fails recall by 0.9 percentage points. The only row that cleared recall, `global768/rw0`, required 9,525,502 candidates and p50 56.486 ms, so it scans far too much and does not improve latency enough.
