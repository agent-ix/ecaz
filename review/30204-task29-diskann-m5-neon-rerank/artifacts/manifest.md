# Artifact Manifest

Packet: `review/30204-task29-diskann-m5-neon-rerank`

Lane: ec_diskann Apple-Silicon NEON exact rerank kernel A/B.

Hardware: Apple M5 (`aarch64-apple-darwin25.2.0`), local PG18 18.3 (Homebrew),
socket `/Users/peter/.pgrx`, port `28818`.

Storage format: `ec_diskann` `pq_fastscan` (default), reloptions
`graph_degree=32`, `build_list_size=100`, `alpha=1.2`. `rerank_budget`
left at the index reloption default (`64`).

Rerank mode: heap-f32 exact rerank (the path through
`exact_heap_rerank_distance(...) -> ambuild::source_inner_product(...)`).

Surface: isolated one-index-per-table prefix `m5_diskann_synth10k`
in database `postgres`.

Cache state: warm local run; both arms used the same on-disk index
that was built once on the scalar binary.

## Code SHAs

- scalar baseline binary: built from `e5f380a1` (current `origin/main`),
  installed sha256 `fc71290a464ddaefe6559f6485bc6c834be3f3bdc2c0e525cdb74b05779ecb7d`.
- NEON post binary: built from `dceda057`
  (`Add NEON exact rerank inner product to ec_diskann` on branch
  `ec-diskann-apple-neon-rerank`), installed sha256
  `0538822d360075f8d8aac566800d94f19c92310ad728d2e8d49655067d9ae307`.

The on-disk index `m5_diskann_synth10k_idx` was built once under the
scalar binary at `e5f380a1` (build elapsed `204.49s`) and reused under
the NEON binary without rebuild.

## Fixture

Synthetic unit-sphere corpus generated with `ecaz corpus generate`:

- corpus: `fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv`
  (`10000 × dim 1536`, seed `42`, sha256
  `ccd9a13cdf99eded145fe92ba65d135a57495b55513444caf35c54d5bdcc6f2f`).
- queries: `fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv`
  (`200 × dim 1536`, seed `7`, sha256
  `a93d23a9b17e3a9ebd8912c9610863453f1ac95728efda3036beef96b7df933f`).

This fixture is NOT a faithful Task 29 substitute. The Task 29 lane used
real DBpedia-style 10k embeddings (`target/real-corpus/ec_hnsw_real_10k`),
which were not available locally on this Apple machine. Synthetic
unit-sphere vectors at dim 1536 are nearly equidistant, so recall@10
falls to `0.16-0.33` and per-query latency is dominated by scan +
heap-fetch overhead rather than the exact rerank kernel. Treat the
numbers below as a kernel correctness + smoke-A/B, not a quality-lane
promotion signal.

## Commands

Generate corpus + queries:

```
/Users/peter/.cargo/bin/ecaz --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/corpus-generate.log corpus generate --output fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv --n 10000 --dim 1536 --seed 42 --kind corpus
/Users/peter/.cargo/bin/ecaz --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/queries-generate.log corpus generate --output fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv --n 200 --dim 1536 --seed 7 --kind queries
```

Install scalar baseline + load corpus + build diskann index:

```
/Users/peter/.cargo/bin/ecaz --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/install-pg18-scalar.log dev install ecaz-pg-test --pg 18
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/load-diskann.log corpus load --prefix m5_diskann_synth10k --corpus-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv --queries-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv --profile ec_diskann --bits 4 --seed 42 --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2
```

Scalar baseline benchmarks:

```
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/recall-scalar-cli.log bench recall --prefix m5_diskann_synth10k --profile ec_diskann --k 10 --sweep 64,200,800 --force-index --truth-cache-file review/30204-task29-diskann-m5-neon-rerank/artifacts/truth_synth10k_k10.json --log-output review/30204-task29-diskann-m5-neon-rerank/artifacts/recall-scalar-table.log
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/latency-scalar-cli.log bench latency --prefix m5_diskann_synth10k --profile ec_diskann --k 10 --sweep 64,200,800 --iterations 200 --concurrency 1 --force-index --sample-backend-memory --log-output review/30204-task29-diskann-m5-neon-rerank/artifacts/latency-scalar-table.log
```

Install NEON branch + post benchmarks (same on-disk index):

```
/Users/peter/.cargo/bin/ecaz --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/install-pg18-neon.log dev install ecaz-pg-test --pg 18
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/recall-neon-cli.log bench recall --prefix m5_diskann_synth10k --profile ec_diskann --k 10 --sweep 64,200,800 --force-index --truth-cache-file review/30204-task29-diskann-m5-neon-rerank/artifacts/truth_synth10k_k10.json --log-output review/30204-task29-diskann-m5-neon-rerank/artifacts/recall-neon-table.log
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 --log-file review/30204-task29-diskann-m5-neon-rerank/artifacts/latency-neon-cli.log bench latency --prefix m5_diskann_synth10k --profile ec_diskann --k 10 --sweep 64,200,800 --iterations 200 --concurrency 1 --force-index --sample-backend-memory --log-output review/30204-task29-diskann-m5-neon-rerank/artifacts/latency-neon-table.log
```

## Artifacts

### `recall-scalar-table.log` / `recall-neon-table.log`

Recall@10, NDCG@10, mean q-time per L. Identical recall and NDCG across
both arms confirms NEON kernel parity with scalar.

| L | scalar recall@10 | NEON recall@10 | scalar NDCG@10 | NEON NDCG@10 | scalar mean | NEON mean |
|---:|---:|---:|---:|---:|---:|---:|
| 64 | 0.1650 | 0.1650 | 0.8298 | 0.8298 | 2.18 ms | 3.74 ms |
| 200 | 0.2665 | 0.2665 | 0.8811 | 0.8811 | 2.58 ms | 2.64 ms |
| 800 | 0.3260 | 0.3260 | 0.9036 | 0.9036 | 3.57 ms | 3.65 ms |

Note: recall mean q-times are computed from a single 200-query pass and
are noisier than the latency bench below; the L=64 recall mean is the
single-pass mean, see latency table for the 200-iteration pass.

### `latency-scalar-table.log` / `latency-neon-table.log`

Per-L latency (200 iterations, concurrency=1, `--force-index`).

| L | scalar mean | scalar p50 | scalar p95 | scalar p99 | NEON mean | NEON p50 | NEON p95 | NEON p99 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.23 ms | 2.18 ms | 2.51 ms | 2.92 ms | 2.27 ms | 2.22 ms | 2.57 ms | 2.76 ms |
| 200 | 2.96 ms | 2.86 ms | 3.48 ms | 4.49 ms | 2.78 ms | 2.73 ms | 3.08 ms | 3.20 ms |
| 800 | 4.12 ms | 4.03 ms | 4.77 ms | 5.21 ms | 3.83 ms | 3.77 ms | 4.13 ms | 4.88 ms |

Per-arm stddev: scalar 0.29/0.39/0.42 ms, NEON 0.25/0.21/0.30 ms.

### Other logs

- `corpus-generate.log`, `queries-generate.log`: corpus generation.
- `install-pg18-scalar.log`, `install-pg18-neon.log`: backend install
  with sha256 of the installed `ecaz.dylib`.
- `load-diskann.log`: corpus load + diskann index build.
- `truth_synth10k_k10.json`: exact top-10 ground truth used by both
  recall arms.
