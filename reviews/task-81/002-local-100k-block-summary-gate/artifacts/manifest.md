# Task 81 Packet 002 Artifact Manifest

- Task bucket: `reviews/task-81/`
- Packet: `reviews/task-81/002-local-100k-block-summary-gate/`
- Head SHA: `60a6bc136eaa6efd804de136330187be6a035c2e`
- Captured: `2026-06-04 19:50:50 PDT`
- Lane: local PG18 / Intel local / `ec_real_100k` / RaBitQ
- Fixture: existing shared-table surface in database `task79_spire_candidate_surface`
- Corpus table: `task79_surface_100k_corpus`
- Query table: `task79_surface_100k_queries`
- Index: `task79_surface_100k_idx`
- Standard runner: `ecaz bench suite`
- Suite config: `reviews/task-81/002-local-100k-block-summary-gate/suite-local-100k-block-summary-gate.json`
- Suite manifest: `reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-manifest.json`
- Results JSONL: `reviews/task-81/002-local-100k-block-summary-gate/artifacts/results.jsonl`
- Storage format: `rabitq`
- Rerank mode: `rerank_width=25`
- Query count: `200`
- Isolation mode: shared-table existing local 100k surface

## Commands

Audit:

```sh
script -q -c "target/debug/ecaz bench suite audit --config reviews/task-81/002-local-100k-block-summary-gate/suite-local-100k-block-summary-gate.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818" reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-audit.log
```

Run:

```sh
target/debug/ecaz bench suite run --config reviews/task-81/002-local-100k-block-summary-gate/suite-local-100k-block-summary-gate.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --manifest-output reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-manifest.json --log-file reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-run.log
```

Status:

```sh
target/debug/ecaz bench suite status --manifest reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-manifest.json --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-status.log
```

Report:

```sh
target/debug/ecaz bench suite report --manifest reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-manifest.json --results-output reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-report-results.jsonl --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 --log-file reviews/task-81/002-local-100k-block-summary-gate/artifacts/suite-report.log
```

## Artifacts

- `suite-audit.log`: suite audit, passed for four steps.
- `suite-run.log`: full suite stdout/stderr, completed four steps.
- `suite-status.log`: completion state, `completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `suite-report.log`: markdown report from the suite manifest.
- `suite-report-results.jsonl`: normalized result rows parsed from artifacts.
- `suite-manifest.json`: structured suite manifest for the full run.
- `results.jsonl`: normalized parsed result rows written by the suite runner.
- `precheck-task81-local-100k-surface.log`: fixture, index, extension, and diagnostic SQL surface precheck.
- `pipeline-100k-rabitq-full-leaf-nprobe96.log`: full-leaf comparator.
- `funnel-100k-rabitq-full-leaf-nprobe96.jsonl`: full-leaf funnel output.
- `pipeline-100k-rabitq-block-summary-global1152.log`: block-summary pruning row.
- `funnel-100k-rabitq-block-summary-global1152.jsonl`: block-summary funnel output.
- `diagnostics-100k-rabitq-block-summary-global1152.log`: per-leaf aggregate block counter and timing diagnostic.

## Key Results

Full-leaf comparator:

- `nprobe=96`
- `candidate_sum=15506227`
- `latency_p50=65.106 ms`
- `latency_p95=77.548 ms`
- `latency_p99=97.520 ms`
- `recall@k=0.9975`

Block-summary pruning:

- `nprobe=96`
- `ec_spire.leaf_block_pruning_max_global_blocks=1152`
- `candidate_sum=3673383`
- `latency_p50=33.472 ms`
- `latency_p95=40.328 ms`
- `latency_p99=46.322 ms`
- `recall@k=0.9940`

Block diagnostic aggregate:

- `query_count=200`
- `candidate_rows=3673383`
- `blocks_available=977202`
- `blocks_selected=230400`
- `blocks_skipped=746802`
- `summary_bytes=2323638996`
- `row_bytes=12641349328`
- `summary_score_nanos=1282625876`
- `row_score_nanos=1636663557`
- `candidate_score_nanos=2919289433`

## Gate Readout

- Recall gate: pass, `0.9940 >= 0.9925`.
- Candidate gate: pass, `3,673,383 <= 4,000,000` over the 100k / 200-query lane.
- Local latency gate: pass, p50 `33.472 ms <= 45 ms`.
- Comparator: block summaries reduce candidates by `76.31%` vs full leaf (`3,673,383` vs `15,506,227`) while keeping recall above the Task 81 floor.
