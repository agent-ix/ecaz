# Task 79 Packet 044 Manifest: RaBitQ Deferred Leaf Segment Decode

- head SHA at packet creation: `5aaa88cdfd904a3e0ee91dbfa40e918494d0f8b7`
- source checkpoint under review: `129a00fab` (`Defer SPIRE leaf segment decoding after block pruning`)
- later branch-only feedback commit: `5aaa88cdf` (`Reviewer feedback: Task 79/043 route-prior global cap`)
- packet commit before PG18 clippy addendum: `f6295a34f` (`Add Task 79 deferred leaf segment decode packet`)
- branch: `task-79-spire-candidate-surface-reduction`
- task bucket: `reviews/task-79/044-rabitq-deferred-leaf-segment-decode/`
- timestamp: `2026-06-02T07:42:12-07:00`
- environment: local PG18, socket `/home/peter/.pgrx`, database `task79_spire_candidate_surface`
- AWS: not used
- installed backend SHA256: `c7bae4e16804615d8e853b7308d782c5a38741711e62fbcf1a68e73edc645ee8`
- primary storage format: `rabitq`
- comparison storage format: `turboquant`
- fixture: `task79_surface_100k`, 100k real corpus/query surface, 200-query benchmark shape
- RaBitQ surface isolation: shared local task 79 corpus/query tables with active index `task79_surface_100k_idx`
- TurboQuant surface isolation: copied local tables `task79_surface_100k_turboquant_*` with index `task79_surface_100k_turboquant_idx`
- RaBitQ index shape: `nlists=128`, `recursive_fanout=8`, `boundary_replica_count=0`, top graph enabled with degree 32 and search list size 96
- RaBitQ leaf block shape: block16, k=3 summaries from the resident Task 79 local index
- rerank mode: heap rerank width 25, recall@10 enabled against `target/real-corpus/staged-task50/ec_real_100k_corpus.tsv`
- routing: `nprobe=96`, adaptive nprobe off
- selector: full k3 summary scoring, per-leaf cap disabled, global block caps varied, radius weight 0.25, route prior 0.0
- winning route classification: deferred subleaf/storage-format design. The
  winning route still uses k=3 RaBitQ block-summary global selection; the new
  source change avoids decoding non-selected assignment segments after that
  summary pass.

## Code Change

- `src/am/ec_spire/storage/*`: adds summary-only leaf reads and selected-row-range segment reads for local and relation object stores.
- `src/am/ec_spire/scan/*`: changes non-sampled global block pruning to read leaf summaries first, select global block ranges, then decode assignment segments only for selected ranges. The sampled-global-probe path stays on the previous full-object path.
- `src/am/ec_spire/storage/tests/leaf.rs`: adds `leaf_partition_object_v2_selected_segment_reader_filters_by_row_range`.

## Commands

- source tests:
  `script -q -c "cargo test leaf_partition_object_v2_selected_segment_reader_filters_by_row_range --no-default-features --features pg18" reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/cargo-test-selected-segment-reader.log`
- scan selector tests:
  `script -q -c "cargo test global_leaf_block_row_ranges --no-default-features --features pg18" reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/cargo-test-global-block-row-ranges.log`
- clippy:
  `script -q -c "cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings" reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/cargo-clippy-pg18.log`
- formatting:
  `cargo fmt --check`
- PG18 install:
  `script -q -c "target/debug/ecaz dev install ecaz-pg-test --pg 18" reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/install-deferred-segments-ecaz-pg18.log`
- PG18 restart:
  `script -q -c "/home/peter/.pgrx/18.3/pgrx-install/bin/pg_ctl -D /home/peter/.pgrx/data-18 -l /home/peter/.pgrx/pg18-current.log restart -m fast" reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/pg18-restart-deferred-segments.log`
- RaBitQ suite audit:
  `script -q -c "target/debug/ecaz bench suite audit --config reviews/task-79/044-rabitq-deferred-leaf-segment-decode/suite-rabitq-deferred-leaf-segment-decode.json" reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-audit.log`
- RaBitQ suite low caps:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/044-rabitq-deferred-leaf-segment-decode/suite-rabitq-deferred-leaf-segment-decode.json --log-file reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-run.log`
- RaBitQ suite high caps:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/044-rabitq-deferred-leaf-segment-decode/suite-rabitq-deferred-leaf-segment-decode.json --only pipeline-100k-rabitq-k3-deferred-global1024-rw025 --only pipeline-100k-rabitq-k3-deferred-global1152-rw025 --only pipeline-100k-rabitq-k3-deferred-global1216-rw025 --manifest-output reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-manifest-high-caps.json --results-output reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/results-high-caps.jsonl --log-file reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-run-high-caps.log`
- TurboQuant isolated comparison:
  `target/debug/ecaz --database task79_spire_candidate_surface --host /home/peter/.pgrx --port 28818 bench suite run --config reviews/task-79/044-rabitq-deferred-leaf-segment-decode/suite-turboquant-comparison.json --manifest-output reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-manifest-turboquant-isolated.json --results-output reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/results-turboquant-isolated.jsonl --log-file reviews/task-79/044-rabitq-deferred-leaf-segment-decode/artifacts/suite-run-turboquant-isolated.log`

## Artifacts

- `suite-rabitq-deferred-leaf-segment-decode.json`: checked-in SuiteConfig for RaBitQ local rows.
- `suite-turboquant-comparison.json`: checked-in SuiteConfig for isolated TurboQuant comparison.
- `artifacts/cargo-test-selected-segment-reader.log`: focused storage test log. Key line: `1 passed; 0 failed`.
- `artifacts/cargo-test-global-block-row-ranges.log`: focused scan selector test log. Key line: `4 passed; 0 failed`.
- `artifacts/cargo-clippy-pg18.log`: PG18 clippy log. Key line: `Finished dev profile`.
- `artifacts/install-deferred-segments-ecaz-pg18.log`: local PG18 install log. Key line: `sha256=c7bae4e16804615d8e853b7308d782c5a38741711e62fbcf1a68e73edc645ee8`.
- `artifacts/pg18-restart-deferred-segments.log`: local PG18 restart log.
- `artifacts/precheck-existing-task79-k3-index.log`: RaBitQ corpus/query/index precheck. Key lines: `corpus_rows=100000`, `query_rows=1000`, `task79_surface_100k_idx`.
- `artifacts/precheck-task79-indexes.log`: confirms the main task surface initially had only `task79_surface_100k_idx`.
- `artifacts/suite-audit*.log`: suite audit logs.
- `artifacts/suite-dry-run.log` and `artifacts/suite-dry-run-manifest.json`: RaBitQ dry-run expansion.
- `artifacts/suite-manifest.json`: completed RaBitQ low-cap run, 5 steps.
- `artifacts/suite-manifest-high-caps.json`: completed RaBitQ high-cap selected-step run, 3 completed / 5 skipped by selector.
- `artifacts/suite-manifest-turboquant-isolated.json`: completed isolated TurboQuant comparison run, 2 steps.
- `artifacts/results.jsonl`, `artifacts/results-high-caps.jsonl`, `artifacts/results-turboquant-isolated.jsonl`: normalized suite result streams. For `pipeline-100k-rabitq-k3-deferred-global1152-rw025`, candidate and heap-rerank rows record `ready_sum=5000`; the endpoint result row records `returned_sum=2000`.
- `artifacts/suite-status-*.log`: suite status outputs.
- `artifacts/suite-report-*.log` and `artifacts/report-results-*.jsonl`: suite reports.
- `artifacts/pipeline-100k-rabitq-k3-deferred-*.log`: per-row RaBitQ pipeline logs.
- `artifacts/funnel-100k-rabitq-k3-deferred-*.jsonl`: per-row RaBitQ funnel output.
- `artifacts/prepare-100k-turboquant-n128-f8-b0-tg96-block16.log`: isolated TurboQuant surface/index setup.
- `artifacts/pipeline-100k-turboquant-block16-global1152-rw025.log`: TurboQuant comparison pipeline.
- `artifacts/funnel-100k-turboquant-block16-global1152-rw025.jsonl`: TurboQuant comparison funnel output.
- `artifacts/compact-results.tsv`: compact table cited by `request.md`.
- `artifacts/suite-run-expanded.log`: failed resume attempt after SuiteConfig hash changed; kept as packet-local provenance for why high caps used a selected-step manifest.
- `artifacts/suite-run-turboquant.log`, `artifacts/suite-manifest-turboquant.json`, and `artifacts/rebuild-100k-turboquant-n128-f8-b0-tg96-block16.log`: failed same-table TurboQuant comparison attempt; relation context failed with two SPIRE indexes on the same corpus table, so the accepted comparison uses the isolated surface artifacts instead.

## Key Results

Compact result table:

```text
row	storage	global_blocks	candidates	latency_p50_ms	latency_p95_ms	latency_p99_ms	recall_at_10	returned_sum	gate
task78_baseline	rabitq	0	15506227	60.256	NA	NA	0.9975	2000	baseline
packet043_block16_reference	rabitq	1216	3877368	56.145	65.566	NA	0.9940	NA	reference_before_deferred_decode
deferred_global704	rabitq	704	2245070	29.221	33.111	41.779	0.9845	2000	fail_recall
deferred_global720	rabitq	720	2296079	29.427	33.589	39.505	0.9845	2000	fail_recall
deferred_global736	rabitq	736	2347078	29.506	32.939	40.749	0.9850	2000	fail_recall
deferred_global768	rabitq	768	2449116	29.962	35.176	37.495	0.9865	2000	fail_recall
deferred_global1024	rabitq	1024	3265373	33.023	39.633	43.467	0.9920	2000	fail_recall_by_0.0005
deferred_global1152	rabitq	1152	3673383	35.293	40.600	47.990	0.9940	2000	pass_best
deferred_global1216	rabitq	1216	3877368	35.812	41.836	44.193	0.9940	2000	pass_reference_cap
turboquant_global1152	turboquant	1152	15506227	141.561	153.951	163.995	0.9975	2000	comparison_not_candidate_reduced
```

Interpretation:

- Best RaBitQ row is `global1152`: `3,673,383` candidates, retained `5,000`, returned `2,000`, p50 `35.293 ms`, recall@10 `0.9940`.
- Versus Task 78 RaBitQ nprobe96, the best row cuts candidates by `76.3%` and p50 by `41.4%` while staying above the `0.9925` recall floor.
- Versus packet 043's block16/global1216 reference, the same candidate/recall row improves p50 from `56.145 ms` to `35.812 ms`.
- Lower caps are now very fast but still fail recall; `global1024` misses the recall floor by only `0.0005`.
- TurboQuant comparison confirms this candidate-surface path is RaBitQ-specific: with `storage_format=turboquant`, global1152 did not reduce candidates and scanned `15,506,227` rows.
