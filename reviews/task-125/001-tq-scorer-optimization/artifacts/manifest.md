# Task 125-129 TurboQuant Scorer Optimization Artifacts

- task bucket: `reviews/task-125/001-tq-scorer-optimization`
- code commits:
  - `da1c79a0c Optimize TurboQuant scoring batch path`
  - `9d8ce1da12 Enable sparse TurboQuant bound pruning on NEON`
  - `3d66bdcf3 Compact TurboQuant LUT scorer table`
- base commit: `6799686af9e9adf13332bd4ec6e19b60e7ceb80e`
- lane: local PG18, aarch64/NEON, `tqvector_bench`
- fixture: staged real corpus, `ec_ivf`, `storage_format=turboquant`, `bits=4`, `seed=42`, `nprobe=32`
- runner: `target/release/ecaz bench suite`
- timestamp: 2026-07-01T07:52:03Z
- isolation: existing one-index-per-prefix tables were reused; load steps skipped reload/rebuild when reloptions matched.

## Suite Config

- `tq-ivf-suite.json`
- sha256: `b258f49b2e712dcfd60e2d991656e6d338e37dec26b112158cf4075fbdc7e0ad`

## Commands

Baseline:

```sh
target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-125/001-tq-scorer-optimization/artifacts/tq-ivf-suite.json --artifact-dir reviews/task-125/001-tq-scorer-optimization/artifacts/baseline --manifest-output reviews/task-125/001-tq-scorer-optimization/artifacts/baseline-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/baseline-results.jsonl
target/release/ecaz bench suite report --manifest reviews/task-125/001-tq-scorer-optimization/artifacts/baseline-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/baseline-results-report.jsonl
```

Final candidate:

```sh
target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-125/001-tq-scorer-optimization/artifacts/tq-ivf-suite.json --artifact-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final --manifest-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final-results.jsonl
target/release/ecaz bench suite report --manifest reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-final-results-report.jsonl
```

Sparse Task 127 candidate:

```sh
target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-125/001-tq-scorer-optimization/artifacts/tq-ivf-suite.json --artifact-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse --manifest-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse-results.jsonl
target/release/ecaz bench suite report --manifest reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-t127-sparse-results-report.jsonl
```

Final int16 LUT candidate:

```sh
target/release/ecaz --database tqvector_bench --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-125/001-tq-scorer-optimization/artifacts/tq-ivf-suite.json --artifact-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-int16-lut --manifest-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-int16-lut-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-int16-lut-results.jsonl
target/release/ecaz bench suite report --manifest reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-int16-lut-suite-manifest.json --results-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-int16-lut-results-report.jsonl
```

## Key Results

Baseline -> final candidate:

- 10k recall: `0.9734 -> 0.9734`; latency mean `1.26 ms -> 1.20 ms`; p50 `1.22 ms -> 1.20 ms`; p95 `1.47 ms -> 1.27 ms`; TurboQuant NEON kernel `45.450584 ms -> 45.274061 ms`; index storage `1028.1 B/row -> 1028.1 B/row`; total storage `17703.7 B/row -> 17705.4 B/row`.
- 50k recall: `0.9521 -> 0.9521`; latency mean `2.55 ms -> 2.55 ms`; p50 `2.51 ms -> 2.51 ms`; p95 `2.83 ms -> 2.81 ms`; TurboQuant NEON kernel `75.372179 ms -> 75.735667 ms`; index storage `964.7 B/row -> 964.7 B/row`; total storage `17634.1 B/row -> 17634.9 B/row`.
- 100k recall: `0.8969 -> 0.8969`; latency mean `3.91 ms -> 3.85 ms`; p50 `3.88 ms -> 3.75 ms`; p95 `4.68 ms -> 4.73 ms`; TurboQuant NEON kernel `75.170506 ms -> 74.262683 ms`; index storage `948.2 B/row -> 948.2 B/row`; total storage `17616.8 B/row -> 17617.5 B/row`.

Baseline -> sparse Task 127 candidate:

- 10k recall: `0.9734 -> 0.9734`; latency mean `1.26 ms -> 1.22 ms`; p50 `1.22 ms -> 1.21 ms`; p95 `1.47 ms -> 1.33 ms`; TurboQuant NEON kernel `45.450584 ms -> 46.240597 ms`; index storage `1028.1 B/row -> 1028.1 B/row`; total storage `17703.7 B/row -> 17705.4 B/row`.
- 50k recall: `0.9521 -> 0.9521`; latency mean `2.55 ms -> 2.58 ms`; p50 `2.51 ms -> 2.55 ms`; p95 `2.83 ms -> 2.84 ms`; TurboQuant NEON kernel `75.372179 ms -> 76.955763 ms`; index storage `964.7 B/row -> 964.7 B/row`; total storage `17634.1 B/row -> 17634.9 B/row`.
- 100k recall: `0.8969 -> 0.8969`; latency mean `3.91 ms -> 3.86 ms`; p50 `3.88 ms -> 3.78 ms`; p95 `4.68 ms -> 4.68 ms`; TurboQuant NEON kernel `75.170506 ms -> 74.883089 ms`; index storage `948.2 B/row -> 948.2 B/row`; total storage `17616.8 B/row -> 17617.5 B/row`.

Sparse Task 127 candidate -> final int16 LUT candidate:

- 10k recall: `0.9734 -> 0.9734`; latency mean `1.22 ms -> 0.88 ms`; p50 `1.21 ms -> 0.85 ms`; p95 `1.33 ms -> 0.99 ms`; TurboQuant NEON kernel `46.240597 ms -> 23.924550 ms`; index storage `1028.1 B/row -> 1028.1 B/row`; total storage `17705.4 B/row -> 17705.4 B/row`.
- 50k recall: `0.9521 -> 0.9521`; latency mean `2.58 ms -> 1.88 ms`; p50 `2.55 ms -> 1.85 ms`; p95 `2.84 ms -> 2.03 ms`; TurboQuant NEON kernel `76.955763 ms -> 41.573022 ms`; index storage `964.7 B/row -> 964.7 B/row`; total storage `17634.9 B/row -> 17634.9 B/row`.
- 100k recall: `0.8969 -> 0.8969`; latency mean `3.86 ms -> 2.73 ms`; p50 `3.78 ms -> 2.64 ms`; p95 `4.68 ms -> 3.32 ms`; TurboQuant NEON kernel `74.883089 ms -> 38.896097 ms`; index storage `948.2 B/row -> 948.2 B/row`; total storage `17617.5 B/row -> 17617.5 B/row`.

## Task 127 Activation

Task 127 is enabled for TurboQuant when the active ISA is NEON. The bounded scorer checks suffix bounds at 512-dimension checkpoints and at the final dimension so the common no-prune case does not pay a per-32-dimension bound-check cost. Non-NEON sessions return `false` from the bounded TurboQuant batch attempt and continue through the existing unbounded batch scorer.

An earlier all-chunk activation attempt is intentionally not part of the review evidence. It regressed the 10k latency run to `7.42 ms` mean and `284.974750 ms` TurboQuant kernel time before the suite was stopped; the sparse NEON-only candidate above replaces that approach.

## Artifact Index

- Baseline suite: `baseline-suite-manifest.json`, `baseline-results.jsonl`, `baseline-results-report.jsonl`, `baseline/*.log`
- Final suite: `candidate-final-suite-manifest.json`, `candidate-final-results.jsonl`, `candidate-final-results-report.jsonl`, `candidate-final/*.log`
- Sparse Task 127 suite: `candidate-t127-sparse-suite-manifest.json`, `candidate-t127-sparse-results.jsonl`, `candidate-t127-sparse-results-report.jsonl`, `candidate-t127-sparse/*.log`
- Final int16 LUT suite: `candidate-int16-lut-suite-manifest.json`, `candidate-int16-lut-results.jsonl`, `candidate-int16-lut-results-report.jsonl`, `candidate-int16-lut/*.log`
- Closeout tiled suite: `candidate-closeout-tiled/suite-manifest.json`, `candidate-closeout-tiled/results.jsonl`, `candidate-closeout-tiled/*.log`
- Task 126 width profile: `task126-width-profile.log`
- Ignored and not committed: `*/truth-cache/`

## Closeout Update: 2026-07-01

- code commit: `96782e209010a70538e94c63dd46e8b2dd54cec2`
- lane: local PG18, aarch64/NEON, `tqvector_bench`
- fixture: staged real corpus, `ec_ivf`, `storage_format=turboquant`, `bits=4`, `seed=42`, `nprobe=32`
- isolation: existing one-index-per-prefix tables were reused; load steps skipped reload/rebuild when reloptions matched.

Commands:

```sh
cargo fmt --check
cargo check -p ecaz --lib
cargo test -p ecaz --lib lut32_tiled_batch_matches_scalar_tail_bits_across_widths_and_dims -- --test-threads=1
cargo test -p ecaz --lib turboquant_lut_bounded_batch_keeps_and_prunes -- --test-threads=1
cargo test -p ecaz --lib lut32_ -- --test-threads=1
cargo test -p ecaz --lib turboquant_no_qjl -- --test-threads=1
cargo test -p ecaz-cli block_kernel_counter_lines_include_transition_formats -- --test-threads=1
ECAZ_TQ_BATCH_WIDTH_PROFILE_LOG=reviews/task-125/001-tq-scorer-optimization/artifacts/task126-width-profile.log ECAZ_TQ_BATCH_WIDTH_PROFILE_CANDIDATES=20000 cargo test -p ecaz --lib task124_profile_tq_no_qjl_flush_widths -- --ignored --nocapture --test-threads=1
cargo build --release -p ecaz
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config
cargo build --release -p ecaz-cli
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench suite run --config reviews/task-125/001-tq-scorer-optimization/artifacts/tq-ivf-suite.json --artifact-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench latency --prefix task125_tq_ivf_real10k --profile ec_ivf --k 10 --concurrency 1 --iterations 64 --sweep 32 --bits 4 --seed 42 --force-index --sample-backend-memory --cache-state task125_tq_ivf_real10k_warm --task87-candidate-batch-counters --memory-sample-interval-ms 25 --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/latency-ivf-tq-real10k-prune-counters.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench latency --prefix task125_tq_ivf_real50k --profile ec_ivf --k 10 --concurrency 1 --iterations 48 --sweep 32 --bits 4 --seed 42 --force-index --sample-backend-memory --cache-state task125_tq_ivf_real50k_warm --task87-candidate-batch-counters --memory-sample-interval-ms 25 --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/latency-ivf-tq-real50k-prune-counters.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench latency --prefix task125_tq_ivf_real100k --profile ec_ivf --k 10 --concurrency 1 --iterations 32 --sweep 32 --bits 4 --seed 42 --force-index --sample-backend-memory --cache-state task125_tq_ivf_real100k_warm --task87-candidate-batch-counters --memory-sample-interval-ms 25 --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/latency-ivf-tq-real100k-prune-counters.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/load-ivf-tq-real10k-heap-rerank.log corpus load --prefix task125_tq_ivf_real10k_heap --profile ec_ivf --corpus-file data/staged-current/ec_real_10k_corpus.tsv --queries-file data/staged-current/ec_real_10k_queries.tsv --manifest-file data/staged-current/ec_real_10k_manifest.json --allow-manifest-mismatch --bits 4 --seed 42 --storage-format turboquant --reloption rerank=heap_f32 --reloption rerank_width=100
target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/load-ivf-tq-real50k-heap-rerank.log corpus load --prefix task125_tq_ivf_real50k_heap --profile ec_ivf --corpus-file data/staged-current/ec_real_50k_corpus.tsv --queries-file data/staged-current/ec_real_50k_queries.tsv --manifest-file data/staged-current/ec_real_50k_manifest.json --allow-manifest-mismatch --bits 4 --seed 42 --storage-format turboquant --reloption rerank=heap_f32 --reloption rerank_width=100
target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/load-ivf-tq-real100k-heap-rerank.log corpus load --prefix task125_tq_ivf_real100k_heap --profile ec_ivf --corpus-file data/staged-current/ec_real_100k_corpus.tsv --queries-file data/staged-current/ec_real_100k_queries.tsv --manifest-file data/staged-current/ec_real_100k_manifest.json --allow-manifest-mismatch --bits 4 --seed 42 --storage-format turboquant --reloption rerank=heap_f32 --reloption rerank_width=100
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench latency --prefix task125_tq_ivf_real10k_heap --profile ec_ivf --k 10 --concurrency 1 --iterations 64 --sweep 32 --bits 4 --seed 42 --force-index --sample-backend-memory --cache-state task125_tq_ivf_real10k_heap_warm --session-guc ec_ivf.posting_bound_prune=on --task87-candidate-batch-counters --memory-sample-interval-ms 25 --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/latency-ivf-tq-real10k-heap-prune-on.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench latency --prefix task125_tq_ivf_real50k_heap --profile ec_ivf --k 10 --concurrency 1 --iterations 48 --sweep 32 --bits 4 --seed 42 --force-index --sample-backend-memory --cache-state task125_tq_ivf_real50k_heap_warm --session-guc ec_ivf.posting_bound_prune=on --task87-candidate-batch-counters --memory-sample-interval-ms 25 --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/latency-ivf-tq-real50k-heap-prune-on.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench latency --prefix task125_tq_ivf_real100k_heap --profile ec_ivf --k 10 --concurrency 1 --iterations 32 --sweep 32 --bits 4 --seed 42 --force-index --sample-backend-memory --cache-state task125_tq_ivf_real100k_heap_warm --session-guc ec_ivf.posting_bound_prune=on --task87-candidate-batch-counters --memory-sample-interval-ms 25 --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/latency-ivf-tq-real100k-heap-prune-on.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench recall --prefix task125_tq_ivf_real10k_heap --profile ec_ivf --k 10 --sweep 32 --bits 4 --seed 42 --force-index --session-guc ec_ivf.posting_bound_prune=on --truth-corpus-file data/staged-current/ec_real_10k_corpus.tsv --truth-cache-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/truth-cache --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/recall-ivf-tq-real10k-heap-prune-on.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench recall --prefix task125_tq_ivf_real10k_heap --profile ec_ivf --k 10 --sweep 32 --bits 4 --seed 42 --force-index --session-guc ec_ivf.posting_bound_prune=off --truth-corpus-file data/staged-current/ec_real_10k_corpus.tsv --truth-cache-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/truth-cache --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/recall-ivf-tq-real10k-heap-prune-off.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench recall --prefix task125_tq_ivf_real50k_heap --profile ec_ivf --k 10 --sweep 32 --bits 4 --seed 42 --force-index --session-guc ec_ivf.posting_bound_prune=on --truth-corpus-file data/staged-current/ec_real_50k_corpus.tsv --truth-cache-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/truth-cache --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/recall-ivf-tq-real50k-heap-prune-on.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench recall --prefix task125_tq_ivf_real50k_heap --profile ec_ivf --k 10 --sweep 32 --bits 4 --seed 42 --force-index --session-guc ec_ivf.posting_bound_prune=off --truth-corpus-file data/staged-current/ec_real_50k_corpus.tsv --truth-cache-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/truth-cache --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/recall-ivf-tq-real50k-heap-prune-off.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench recall --prefix task125_tq_ivf_real100k_heap --profile ec_ivf --k 10 --sweep 32 --bits 4 --seed 42 --force-index --session-guc ec_ivf.posting_bound_prune=on --truth-corpus-file data/staged-current/ec_real_100k_corpus.tsv --truth-cache-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/truth-cache --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/recall-ivf-tq-real100k-heap-prune-on.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 bench recall --prefix task125_tq_ivf_real100k_heap --profile ec_ivf --k 10 --sweep 32 --bits 4 --seed 42 --force-index --session-guc ec_ivf.posting_bound_prune=off --truth-corpus-file data/staged-current/ec_real_100k_corpus.tsv --truth-cache-dir reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/truth-cache --log-output reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/recall-ivf-tq-real100k-heap-prune-off.log
target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/storage-ivf-tq-real10k-heap-rerank.log bench storage --prefix task125_tq_ivf_real10k_heap
target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/storage-ivf-tq-real50k-heap-rerank.log bench storage --prefix task125_tq_ivf_real50k_heap
target/release/ecaz --host /Users/peter/.pgrx --port 28818 --log-file reviews/task-125/001-tq-scorer-optimization/artifacts/candidate-closeout-tiled/storage-ivf-tq-real100k-heap-rerank.log bench storage --prefix task125_tq_ivf_real100k_heap
```

Closeout suite key result lines:

- 10k: recall `0.9734`, latency mean `0.91 ms`, p50 `0.87 ms`, p95 `1.04 ms`, TurboQuant NEON kernel `24.348035 ms`, index storage `1028.1 B/row`, total storage `17705.4 B/row`.
- 50k: recall `0.9521`, latency mean `1.83 ms`, p50 `1.76 ms`, p95 `2.03 ms`, TurboQuant NEON kernel `39.530475 ms`, index storage `964.7 B/row`, total storage `17634.9 B/row`.
- 100k: recall `0.8969`, latency mean `2.70 ms`, p50 `2.60 ms`, p95 `3.38 ms`, TurboQuant NEON kernel `38.176593 ms`, index storage `948.2 B/row`, total storage `17617.5 B/row`.

Task 126 width profile (`task126-width-profile.log`, debug microprofile):

- width 32: `12662.2 ns/candidate`
- width 64: `11578.1 ns/candidate`
- width 128: `11084.1 ns/candidate`

Task 127 prune-counter evidence:

- `turboquant_lut_bounded_batch_keeps_and_prunes` verifies the bounded TurboQuant scorer records all-kept and all-pruned batches through `lut32_kept_candidates` / `lut32_pruned_candidates`.
- Release IVF latency logs with the updated counter formatter report no bounded scorer dispatch for the current 10k/50k/100k suite shape:
  - 10k: `pruned_candidates=0 kept_candidates=0`, `lut32_pruned_candidates=0 lut32_kept_candidates=0`
  - 50k: `pruned_candidates=0 kept_candidates=0`, `lut32_pruned_candidates=0 lut32_kept_candidates=0`
  - 100k: `pruned_candidates=0 kept_candidates=0`, `lut32_pruned_candidates=0 lut32_kept_candidates=0`
- The standard no-heap suite has no running top-k cutoff, so the production prune path was rechecked on one-index-per-prefix TurboQuant IVF indexes loaded with `rerank=heap_f32,rerank_width=100`. Prune-on and prune-off recall match exactly:
  - 10k: recall `1.0000 -> 1.0000`; latency mean `1.50 ms`; index storage `1028.1 B/row`; `lut32_pruned_candidates=186274`, `lut32_kept_candidates=3301`, bounded prune fraction `98.3%`.
  - 50k: recall `0.9641 -> 0.9641`; latency mean `2.58 ms`; index storage `964.7 B/row`; `lut32_pruned_candidates=320421`, `lut32_kept_candidates=7698`, bounded prune fraction `97.7%`.
  - 100k: recall `0.9268 -> 0.9268`; latency mean `3.80 ms`; index storage `948.2 B/row`; `lut32_pruned_candidates=316516`, `lut32_kept_candidates=7049`, bounded prune fraction `97.8%`.
