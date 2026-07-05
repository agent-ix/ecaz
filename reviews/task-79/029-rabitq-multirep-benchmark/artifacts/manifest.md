# Task 79 Packet 029 Manifest: RaBitQ Multi-Representative Benchmark

- head SHA: `194cc6e07bfd6315b8e494d75a21e9bd1c75e2da`
- implementation commit: `14fcaed21` (`Add RaBitQ multi-representative leaf summaries`)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/029-rabitq-multirep-benchmark/`
- timestamp: `2026-06-02T07:30:35Z`
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
- candidate pruning: global leaf-block cap sweep, `ec_spire.leaf_block_pruning_max_blocks_per_leaf=0`, `global_probe_blocks=0`, `sample_rows_per_block=0`, summary prior weight 0.8, radius weight sweep 0.0/0.25

## Commands

- extension install:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/029-rabitq-multirep-benchmark/artifacts/install-ecaz-pg18.log`
- PG18 restart:
  `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l reviews/task-79/029-rabitq-multirep-benchmark/artifacts/pg18-restart.log restart -m fast`
- suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/029-rabitq-multirep-benchmark/suite-rabitq-multirep-block32.json" reviews/task-79/029-rabitq-multirep-benchmark/artifacts/suite-audit.log`
- suite dry run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --dry-run --config reviews/task-79/029-rabitq-multirep-benchmark/suite-rabitq-multirep-block32.json --manifest-output reviews/task-79/029-rabitq-multirep-benchmark/artifacts/suite-dry-run-manifest.json --log-file reviews/task-79/029-rabitq-multirep-benchmark/artifacts/suite-dry-run.log`
- suite run:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/029-rabitq-multirep-benchmark/suite-rabitq-multirep-block32.json --log-file reviews/task-79/029-rabitq-multirep-benchmark/artifacts/suite-run.log`
- suite status:
  `target/debug/ecaz bench suite status --manifest reviews/task-79/029-rabitq-multirep-benchmark/artifacts/suite-manifest.json --log-file reviews/task-79/029-rabitq-multirep-benchmark/artifacts/suite-status.log`
- suite report:
  `target/debug/ecaz bench suite report --manifest reviews/task-79/029-rabitq-multirep-benchmark/artifacts/suite-manifest.json --results-output reviews/task-79/029-rabitq-multirep-benchmark/artifacts/report-results.jsonl --log-file reviews/task-79/029-rabitq-multirep-benchmark/artifacts/suite-report.log`
- validation:
  `script -q -c "cargo check --no-default-features --features pg18" reviews/task-79/029-rabitq-multirep-benchmark/artifacts/cargo-check-pg18.log`
- validation:
  `script -q -c "cargo test --no-default-features --features pg18 leaf_block" reviews/task-79/029-rabitq-multirep-benchmark/artifacts/cargo-test-leaf-block.log`
- validation:
  `script -q -c "cargo test --no-default-features --features pg18 leaf_partition_object" reviews/task-79/029-rabitq-multirep-benchmark/artifacts/cargo-test-leaf-partition-object.log`

## Artifacts

- `suite-rabitq-multirep-block32.json`: checked-in SuiteConfig for the local RaBitQ multi-representative block32 sweep.
- `artifacts/install-ecaz-pg18.log`: local PG18 extension install log. Backend SHA256: `929efde6155ae01ac72dd90395eea24334c1d496fc48b3f738ce7f52a1c1b15a`.
- `artifacts/pg18-restart.log`: local PG18 restart log.
- `artifacts/suite-audit.log`: suite audit output; 9 steps resolved.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: dry-run expansion for the suite.
- `artifacts/suite-run.log`: raw `ecaz bench suite run` output.
- `artifacts/suite-manifest.json`: suite manifest for the completed local run.
- `artifacts/results.jsonl`: suite-run parsed result stream.
- `artifacts/suite-status.log`: status output, `completed=9 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.
- `artifacts/suite-report.log` and `artifacts/report-results.jsonl`: report output and parsed results.
- `artifacts/compact-results.tsv`: compact candidate/latency/recall table cited by `request.md`.
- `artifacts/cargo-check-pg18.log`: PG18 feature `cargo check`; passed.
- `artifacts/cargo-test-leaf-block.log`: focused leaf block scoring/coverage tests; 10 passed.
- `artifacts/cargo-test-leaf-partition-object.log`: focused storage round-trip/rejection tests; 14 unit tests and 2 fixture tests passed.
- `artifacts/precheck-existing-task79-surface.log`: corpus/query/GUC precheck.
- `artifacts/rebuild-100k-rabitq-n128-f8-b0-tg96-block32-multirep.log`: local RaBitQ index rebuild log. Key line: `ec_spire_ambuild_timing ... total_ms=16533`.
- `artifacts/pipeline-*.log`: per-row pipeline logs with routing, candidate, query metrics, recall, and local production-read profile.
- `artifacts/funnel-*.jsonl`: per-row funnel output.

## Key Results

The compact result table is:

```text
step	global_blocks	radius_weight	candidates	latency_p50_ms	latency_p95_ms	recall_at_10	returned_sum	gate
pipeline-100k-rabitq-n128-f8-b0-tg96-block32-multirep-global0-rw0	0	0	15506227	64.024	77.868	0.9975	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block32-multirep-global512-rw0	512	0	3210821	42.584	48.468	0.9735	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block32-multirep-global512-rw025	512	0.25	3240177	42.714	49.605	0.9795	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block32-multirep-global640-rw0	640	0	4015761	45.782	52.330	0.9840	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block32-multirep-global640-rw025	640	0.25	4050758	44.852	52.726	0.9870	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block32-multirep-global768-rw0	768	0	4820038	46.882	54.576	0.9865	2000	fail
pipeline-100k-rabitq-n128-f8-b0-tg96-block32-multirep-global768-rw025	768	0.25	4860415	48.670	55.112	0.9905	2000	fail
```

Interpretation:

- The best row with both candidate and p50 gates satisfied is `global640/radius0.25`: 4,050,758 candidates, p50 44.852 ms, p95 52.726 ms, recall@10 0.9870.
- The highest-recall capped row is `global768/radius0.25`: 4,860,415 candidates, p50 48.670 ms, p95 55.112 ms, recall@10 0.9905.
- No row satisfies Task 79's first accepted slice gates: recall@10 >= 0.9925, candidates <= 5.2M, and p50 <= 45 ms or 25% better than the 60.256 ms baseline.
- Compared with packet 027's single-representative block32 `global896/radius0.25` row, multi-representative summaries improve candidate count from 5,674,919 to 4,860,415 at the highest-recall capped point, and recall from 0.9900 to 0.9905, but p50 remains too high and recall still misses.
- TurboQuant comparison was intentionally not run because the RaBitQ primary path did not pass the acceptance gates.
- This negative result means two-representative block summaries are not enough by themselves. The next local research step should inspect false-negative target block ranks under the multi-representative score and either calibrate the bound or change candidate selection to recover the remaining recall without raising the global cap.
