# Task 101 Packet 002 Artifact Manifest

- head SHA: `a98d212c5f4336cf780019a9f1b354822b9e5bda`
- task bucket: `reviews/task-101/`
- packet path: `reviews/task-101/002-hnsw-exact-mode-counter-evidence/`
- timestamp: `2026-06-10T14:44:58-07:00`
- lane: local PG18, HNSW, TurboQuant 4-bit, 1536d synthetic 1k corpus / 64 queries
- storage format: `turboquant`
- rerank mode: exact-mode comparison across `full_lut`, `tiled_lut`, and `int8_approx`
- surface isolation: one task-local HNSW fixture prefix (`task101_hnsw_tq_1k`) in database `tqvector_task101`

## Source Check

Artifact:
- `cargo-test-candidate-batch.log`

Command:
`script -q -c "cargo test --lib candidate_batch --no-default-features --features pg18" reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/cargo-test-candidate-batch.log`

Key result:
- `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 2067 filtered out`
- Includes `turboquant_no_qjl_exact_modes_shape_error_scores_nothing_and_record_no_counters`, proving malformed mid-batch exact-mode payloads reject before score writes or counter records.

## Suite Config And Runner Logs

Artifacts:
- `task101-hnsw-exact-mode-counter-suite.json`
- `suite-run.log`
- `suite-audit.log`
- `suite-status.log`
- `suite-report.log`
- `suite-manifest.json`
- `results.jsonl`
- `results-report.jsonl`

Command:
`ecaz bench suite --config reviews/task-101/002-hnsw-exact-mode-counter-evidence/artifacts/task101-hnsw-exact-mode-counter-suite.json`

Key result lines:
- `suite-audit.log`: `[suite:task101-hnsw-exact-mode-counter-suite] audit passed: 15 steps`
- `suite-status.log`: `completed=15 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- `suite-report.log`: `steps: completed 15, failed 0, skipped 0, dry-run 0, missing artifacts 0, stale 0`

## Generated Fixtures

Artifacts:
- `task101_hnsw_tq_1k_corpus.tsv`
- `task101_hnsw_tq_1k_queries.tsv`
- `truth-cache/`

Commands recorded in `suite-run.log`:
- `corpus generate --output .../task101_hnsw_tq_1k_corpus.tsv --n 1000 --dim 1536 --seed 4201 --kind corpus`
- `corpus generate --output .../task101_hnsw_tq_1k_queries.tsv --n 64 --dim 1536 --seed 4202 --kind queries`

## Load

Artifact:
- `load-hnsw-tq-1k.log`

Command recorded in `suite-run.log`:
`corpus load --prefix task101_hnsw_tq_1k --profile ec_hnsw --corpus-file .../task101_hnsw_tq_1k_corpus.tsv --queries-file .../task101_hnsw_tq_1k_queries.tsv --dim 1536 --bits 4 --seed 42 --m 16 --ef-construction 128 --storage-format turboquant`

Key result:
- `results-report.jsonl`: total load phase `seconds=2.620000`

## Exact-Mode Recall And Latency

Artifacts:
- `recall-full_lut-1k-kernel-on.log`
- `latency-full_lut-1k-kernel-on.log`
- `recall-full_lut-1k-kernel-off.log`
- `latency-full_lut-1k-kernel-off.log`
- `recall-tiled_lut-1k-kernel-on.log`
- `latency-tiled_lut-1k-kernel-on.log`
- `recall-tiled_lut-1k-kernel-off.log`
- `latency-tiled_lut-1k-kernel-off.log`
- `recall-int8_approx-1k-kernel-on.log`
- `latency-int8_approx-1k-kernel-on.log`
- `recall-int8_approx-1k-kernel-off.log`
- `latency-int8_approx-1k-kernel-off.log`

Key result lines from `results-report.jsonl`:
- full_lut recall is byte-equal across candidate-batch on/off: `recall@k=0.8375`, `ndcg@k=0.9872`
- tiled_lut recall is byte-equal across candidate-batch on/off: `recall@k=0.8375`, `ndcg@k=0.9872`
- int8_approx recall is byte-equal across candidate-batch on/off: `recall@k=0.8344`, `ndcg@k=0.9869`
- direct exact-mode counter rows are visible:
  - `quant=turboquant`, `cache_state=task101_full_lut_1k_on`
  - `quant=turboquant_tiled_lut`, `cache_state=task101_tiled_lut_1k_on`
  - `quant=turboquant_int8`, `cache_state=task101_int8_approx_1k_on`

Notes:
- The suite ran on local PG18 before packet metadata was recovered from the interrupted session. The focused source check above was rerun after committing `a98d212c5` and is the source-level validation for the code change in this packet.
