# Task 79 Packet 035 Manifest: RaBitQ k=3 Multi-Representative Benchmark

- head SHA: `7a5178f19e59cb2227ef997a530c6e52ee96dce8`
- implementation base commit: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- temporary code patch: `artifacts/k3-cluster-mean.patch` (not committed as production code)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/035-rabitq-k3-multirep-benchmark/`
- timestamp: `2026-06-02T02:15:29-07:00`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- installed k=3 backend SHA256: `096d1790385a4bd22c8568d2e826808278f8ed191f76deec698e8fb737487425`
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: `ec_spire.leaf_block_rows=32`
- summary representation: temporary RaBitQ V4 leaf-block summaries with three cluster-mean representatives per block
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- selector: global block caps 512/640/704/768, radius weight 0.25, per-leaf cap disabled, sampled rescue disabled

## Commands

- focused test:
  `script -q -c "cargo test --no-default-features --features pg18 leaf_block_summaries_cover_rabitq_row_blocks" reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/cargo-test-k3-leaf-block.log`
- temporary patch capture:
  `git diff --output=reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/k3-cluster-mean.patch -- src/am/ec_spire/build/recursive.rs src/am/ec_spire/build/tests/recursive.rs`
- k=3 backend install:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/install-k3-ecaz-pg18.log`
- restart local PG18:
  `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/pg18-restart.log restart -m fast`
- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/035-rabitq-k3-multirep-benchmark/suite-rabitq-k3-multirep-block32.json" reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/035-rabitq-k3-multirep-benchmark/suite-rabitq-k3-multirep-block32.json --manifest-output reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/035-rabitq-k3-multirep-benchmark/suite-rabitq-k3-multirep-block32.json --log-file reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/035-rabitq-k3-multirep-benchmark/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-k3-multirep-block32.json`: checked-in SuiteConfig for the local k=3 sweep.
- `artifacts/k3-cluster-mean.patch`: temporary source patch used for this measurement.
- `artifacts/cargo-test-k3-leaf-block.log`: focused unit test log. Key line: `1 passed; 0 failed`.
- `artifacts/install-k3-ecaz-pg18.log`: patched backend install log. Key line: `sha256=096d1790385a4bd22c8568d2e826808278f8ed191f76deec698e8fb737487425`.
- `artifacts/pg18-restart.log`: local PG18 restart log after the k=3 backend install.
- `artifacts/suite-audit.log`: suite audit output; 6 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck. Key lines: `corpus_rows=100000`, `query_rows=1000`.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-k3.log`: local RaBitQ k=3 index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=17555`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
row	global_blocks	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
k3	512	0.25	3239966	44.708	52.527	0.9855	2000	fail_recall
k3	640	0.25	4050130	49.910	60.820	0.9910	2000	fail_recall_p50
k3	704	0.25	4454827	48.199	56.595	0.9920	2000	fail_recall_p50
k3	768	0.25	4860209	49.252	59.120	0.9925	2000	fail_p50
```

Interpretation:

- k=3 adds real recall signal: global768 reaches the 0.9925 recall gate with 4.86M candidates.
- k=3 does not pass Task 79 because the best recall row has p50 49.252ms, above the 45ms gate.
- The local store overlap object byte sum increased to 13.817GB from the k=2 packet 034/033 value of 13.427GB, reflecting larger summary payloads.
- The next viable path is not more candidate-cap sweeping. It is either score calibration that recovers recall without extra scan-time payload scoring, or a two-stage/reranked summary scoring path that uses richer representatives only on a narrowed block shortlist.
