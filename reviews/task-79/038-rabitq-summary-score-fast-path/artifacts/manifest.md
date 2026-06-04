# Task 79 Packet 038 Manifest: RaBitQ Summary-Score Fast Path

- head SHA: `46d83192ef6f7b2fa88c2f64cffa093ebd88aeaf`
- code checkpoint: `46d83192e` (`Optimize SPIRE leaf block summary scoring`)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/038-rabitq-summary-score-fast-path/`
- timestamp: `2026-06-02T04:17:51-07:00`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- installed fast-path backend SHA256: `210566e905947116d8d9aa6eb718d99368302aa02aca5e17edbc71da96e41a10`
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active index `task79_surface_100k_idx`
- index provenance: reused the local k=3 RaBitQ index rebuilt in packet 037, `reviews/task-79/037-rabitq-two-stage-k3-rescore/artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-k3-two-stage.log`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: `ec_spire.leaf_block_rows=32`
- summary representation: existing RaBitQ V4 leaf-block summaries with three cluster-mean representatives per block from packet 037
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- selector: full k3 summary scoring with global block caps 736/768, radius weight 0.25, no sampled rescue

## Code Change

- `src/am/ec_spire/quantizer/mod.rs`: adds `score_zero_gamma_payload_chunks_max_prevalidated`, a hot-path helper that scores already-validated summary representative chunks through one quantizer branch.
- `src/am/ec_spire/scan/candidates.rs`: adds `SpireLeafBlockSummaryScoreContext` to precompute payload format, payload stride, and RaBitQ radius bonus scale once per selector pass; summary scoring then validates each summary once and uses the prevalidated chunk helper.

## Commands

- formatting:
  `cargo fmt`
- summary scoring tests:
  `script -q -c "cargo test --no-default-features --features pg18 leaf_block_summary" reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/cargo-test-leaf-block-summary.log`
- global selector tests:
  `script -q -c "cargo test --no-default-features --features pg18 select_global_leaf_block_row_ranges" reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/cargo-test-global-block-selection.log`
- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/038-rabitq-summary-score-fast-path/suite-rabitq-summary-score-fast-path.json" reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/038-rabitq-summary-score-fast-path/suite-rabitq-summary-score-fast-path.json --manifest-output reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-dry-run.log`
- fast-path backend install:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/install-fast-path-ecaz-pg18.log`
- restart local PG18:
  `script -q -c "/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l /home/peter/.pgrx/pg18-current.log restart -m fast" reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/pg18-restart-command.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/038-rabitq-summary-score-fast-path/suite-rabitq-summary-score-fast-path.json --log-file reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-manifest.json --log-file reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-manifest.json --results-output reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/report-results.jsonl --log-file reviews/task-79/038-rabitq-summary-score-fast-path/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-summary-score-fast-path.json`: checked-in SuiteConfig for the local fast-path measurement.
- `artifacts/cargo-test-leaf-block-summary.log`: focused block-summary test log. Key line: `2 passed; 0 failed`.
- `artifacts/cargo-test-global-block-selection.log`: focused global selector test log. Key line: `2 passed; 0 failed`.
- `artifacts/install-fast-path-ecaz-pg18.log`: patched backend install log. Key line: `sha256=210566e905947116d8d9aa6eb718d99368302aa02aca5e17edbc71da96e41a10`.
- `artifacts/pg18-restart-command.log`: local PG18 restart command log after installing the fast-path backend.
- `artifacts/suite-audit.log`: suite audit output; 3 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=3 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-k3-index.log`: corpus/query/index/GUC precheck. Key lines: `corpus_rows=100000`, `query_rows=1000`, `task79_surface_100k_idx`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
row	global_blocks	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	packet037_baseline_p50_ms	p50_delta_ms	gate
fast_path	736	4657668	47.909	55.607	0.9925	2000	48.853	-0.944	fail_p50
fast_path	768	4860209	48.989	56.668	0.9925	2000	49.017	-0.028	fail_p50
```

Interpretation:

- The fast path preserves exact block selection behavior: candidate counts and recall match packet 037 full-k3 rows.
- `global736` gets a measurable 0.944 ms p50 improvement, reducing the miss from 3.853 ms to 2.909 ms.
- `global768` is essentially unchanged, so the optimization is not enough to close Task 79 by itself.
- The remaining local work should shift back to reducing selected work, not just scorer dispatch overhead.
