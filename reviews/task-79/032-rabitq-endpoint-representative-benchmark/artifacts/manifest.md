# Task 79 Packet 032 Manifest: RaBitQ Endpoint Representative Benchmark

- head SHA: `c8fdf85ace9c944cf1a2cbe1113b2d3cc0070e3d`
- implementation base: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- benchmark source delta: temporary local patch captured in `artifacts/endpoint-representative-source-diff.patch`
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/032-rabitq-endpoint-representative-benchmark/`
- timestamp: `2026-06-02T08:12:54Z`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- installed backend SHA256: `d5daded78d0b055db46e9a9b19ca727151d2565e55a6b14087ab4dcd69196c69`
- storage format: `rabitq`
- fixture: `task79_surface_100k`, 100k real corpus/query surface
- surface isolation: shared local task 79 corpus/query tables with one active rebuilt index `task79_surface_100k_idx`
- index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- leaf block shape: `ec_spire.leaf_block_rows=32`
- summary representation: RaBitQ V4 leaf-block summaries with two endpoint representatives per block
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- selector: final global block caps 512/640/768, radius weight 0.25, no sampled rescue

## Commands

- install temporary endpoint-representative backend:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/install-ecaz-pg18.log`
- restart local PG18:
  `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/pg18-restart.log restart -m fast`
- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/032-rabitq-endpoint-representative-benchmark/suite-rabitq-endpoint-representative-block32.json" reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/032-rabitq-endpoint-representative-benchmark/suite-rabitq-endpoint-representative-block32.json --manifest-output reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/032-rabitq-endpoint-representative-benchmark/suite-rabitq-endpoint-representative-block32.json --log-file reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/032-rabitq-endpoint-representative-benchmark/artifacts/suite-report.log`

## Artifacts

- `suite-rabitq-endpoint-representative-block32.json`: checked-in SuiteConfig for the endpoint-representative sweep.
- `artifacts/endpoint-representative-source-diff.patch`: exact temporary code delta used for this rejected experiment.
- `artifacts/install-ecaz-pg18.log`: local backend install log. Key line: `sha256=d5daded78d0b055db46e9a9b19ca727151d2565e55a6b14087ab4dcd69196c69`.
- `artifacts/pg18-restart.log`: local PG18 restart log.
- `artifacts/suite-audit.log`: suite audit output; 5 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=5 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-endpoint-reps.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=16119`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
row	final_blocks	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
endpoint	512	0.25	3228407	42.414	49.493	0.8910	2000	fail
endpoint	640	0.25	4034920	44.856	52.945	0.9145	2000	fail
endpoint	768	0.25	4841459	47.202	56.159	0.9310	2000	fail
```

Interpretation:

- Endpoint representatives preserve the candidate and latency shape of the k=2 summary path but destroy recall.
- Compared to packet 029 cluster-mean representatives, recall drops sharply at the same caps: cap512 `0.9795 -> 0.8910`, cap640 `0.9870 -> 0.9145`, cap768 `0.9905 -> 0.9310`.
- This rejects raw farthest-pair endpoints as a candidate-reduction/latency fix. The retained path remains cluster-mean k=2 with a better score calibration or richer but controlled representative strategy.
