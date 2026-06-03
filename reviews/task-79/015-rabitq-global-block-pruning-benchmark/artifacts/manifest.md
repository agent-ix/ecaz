# Task 79 Packet 015 Artifact Manifest

- head SHA at measurement time: `d932841799bac2db65537612f0915460126eb16b`
- code commit measured: `fc2b6ca022ba9e6384807ea2c791c6a784b4a034`
- task bucket: `reviews/task-79/015-rabitq-global-block-pruning-benchmark/`
- packet type: benchmark evidence for RaBitQ global leaf block pruning
- lane / fixture / storage format / rerank mode: intel-local, PG18, `ec_real_100k`, `ec_spire`, RaBitQ, `nlists=128`, `recursive_fanout=8`, `nprobe=96`, `top_graph_search_list_size=96`, `rerank_width=25`, `boundary_replica_count=0`
- isolated one-index-per-table or shared-table surface: shared table `task79_surface_100k_corpus`; one active index `task79_surface_100k_idx` rebuilt in place for block64 summaries
- timestamp: 2026-06-02T01:07:15Z

## Commands

### Suite Validation

- `jq empty reviews/task-79/015-rabitq-global-block-pruning-benchmark/suite-rabitq-global-block-pruning.json`
- `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/015-rabitq-global-block-pruning-benchmark/suite-rabitq-global-block-pruning.json" reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/suite-audit.log`
- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/015-rabitq-global-block-pruning-benchmark/suite-rabitq-global-block-pruning.json --manifest-output reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/suite-dry-run.log`

### Build And Install

- `script -q -c "cargo build -p ecaz-cli" reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/cargo-build-ecaz-cli.log`
- `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/install-ecaz-pg18.log`
- installed backend SHA256: `6a2b4a329061ce35791c9d500aa63ac133a595abb4fa989917522eee40a48969`
- `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/pg18-restart.log restart -m fast`

### Suite Run

- `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/015-rabitq-global-block-pruning-benchmark/suite-rabitq-global-block-pruning.json --log-file reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/suite-run.log`
- `target/debug/ecaz bench suite status --manifest reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/suite-status.log`
- `target/debug/ecaz bench suite report --manifest reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/015-rabitq-global-block-pruning-benchmark/artifacts/suite-report.log`

The final suite status records 8 completed steps, 0 failed, 0 missing artifacts,
and 0 stale steps.

## Artifacts

- `suite-rabitq-global-block-pruning.json`: checked-in `ecaz bench suite` config.
- `artifacts/suite-dry-run-manifest.json`: dry-run manifest.
- `artifacts/suite-manifest.json`: final successful suite manifest.
- `artifacts/suite-run.log`: final suite run stdout/stderr mirror.
- `artifacts/suite-status.log`: final suite status.
- `artifacts/suite-report.log`: final suite report.
- `artifacts/results.jsonl`: raw suite results.
- `artifacts/report-results.jsonl`: report output.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build log.
- `artifacts/install-ecaz-pg18.log`: PG18 extension install log.
- `artifacts/pg18-restart.log`: PG18 restart log.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query count and GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block64-global.log`: RaBitQ block64 index rebuild.
- `artifacts/pipeline-100k-rabitq-n128-f8-b0-tg96-block64-global*.log` and matching `funnel-*.jsonl`: baseline and global block budget pipeline runs.

## Key Results

| Step | Candidate sum | p50 latency | p95 latency | recall@10 |
| --- | ---: | ---: | ---: | ---: |
| global0 baseline | 15,506,227 | 62.122 ms | 73.596 ms | 0.9975 |
| global384 | 4,684,566 | 43.182 ms | 48.897 ms | 0.9675 |
| global400 | 4,882,003 | 43.486 ms | 48.441 ms | 0.9710 |
| global512 | 6,269,044 | 47.527 ms | 54.166 ms | 0.9860 |
| global768 | 9,444,236 | 55.580 ms | 62.348 ms | 0.9925 |
| global1024 | 12,634,733 | 63.926 ms | 73.183 ms | 0.9970 |

All runs had `delta_route_sum=0` and `delta_decode_sum=0`, so delta scoring did not mask leaf-row pruning behavior. All runs retained 5,000 candidates and returned 2,000 rows over 200 queries.

## Interpretation

Global allocation improves the mean-only summary frontier versus per-leaf
allocation, but it does not meet Task 79. The candidate-gate budgets
(`global384`, `global400`) hit the p50 target and score under 5.2M candidates,
but recall stays far below the 0.9925 floor. The first recall-floor point is
`global768`, which reaches recall 0.9925 but still scans 9.44M candidates and
p50 is 55.58 ms.

This attributes the remaining failure to summary semantics rather than budget
allocation. A single mean summary per block cannot identify enough winning
blocks at the required 4M-5.2M candidate surface. The next direct Task 79
attempt should use stronger within-leaf discrimination, for example a tiny
row-sample probe before committing blocks or multi-representative block
summaries.
