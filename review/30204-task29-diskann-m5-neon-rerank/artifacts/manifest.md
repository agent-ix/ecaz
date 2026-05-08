# Artifact Manifest

Packet: `review/30204-task29-diskann-m5-neon-rerank`

Lane: ec_diskann Apple-Silicon NEON exact rerank kernel A/B.

Hardware: Apple M5 (`aarch64-apple-darwin25.2.0`), local PG18 18.3 (Homebrew),
socket `/Users/peter/.pgrx`, port `28818`.

Storage format: `ec_diskann` `pq_fastscan` (default), reloptions
`graph_degree=32`, `build_list_size=100`, `alpha=1.2`. Two `rerank_budget`
points: index reloption default (`64`) on the synth and real fixtures,
and `rerank_budget=800` on the kernel-stress real fixture.

Rerank mode: heap-f32 exact rerank (the path through
`exact_heap_rerank_distance(...) -> ambuild::source_inner_product(...)`).

Surface: isolated one-index-per-table prefixes `m5_diskann_synth10k`,
`m5_diskann_real10k`, `m5_diskann_real10k_w800`, all in database
`postgres`.

Cache state: warm local run; for each prefix both arms used the same
on-disk index that was built once on the scalar binary, except
`m5_diskann_real10k_w800` which was built once on the NEON binary
(only the kernel's runtime cost differs between arms, the index
on-disk shape is the same).

## Code SHAs

- scalar baseline binary: built from `e5f380a1` (current `origin/main`),
  installed sha256 `fc71290a464ddaefe6559f6485bc6c834be3f3bdc2c0e525cdb74b05779ecb7d`.
- NEON post binary: built from `dceda057`
  (`Add NEON exact rerank inner product to ec_diskann` on branch
  `ec-diskann-apple-neon-rerank`), installed sha256
  `0538822d360075f8d8aac566800d94f19c92310ad728d2e8d49655067d9ae307`.

## Fixtures

### `m5_diskann_synth10k` (smoke)

Synthetic unit-sphere corpus generated with `ecaz corpus generate`:

- corpus: `fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv`
  (`10000 x dim 1536`, seed `42`, sha256
  `ccd9a13cdf99eded145fe92ba65d135a57495b55513444caf35c54d5bdcc6f2f`).
- queries: `fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv`
  (`200 x dim 1536`, seed `7`, sha256
  `a93d23a9b17e3a9ebd8912c9610863453f1ac95728efda3036beef96b7df933f`).

Recall@10 falls to `0.16-0.33` here because synthetic high-dim vectors
are nearly equidistant; treat synth numbers as kernel-correctness only.

### `m5_diskann_real10k` (real DBpedia-style 1536d)

The Task 29 `target/real-corpus/ec_hnsw_real_10k` TSVs were not
available locally on this Apple machine. The real-data TSVs were
extracted from the existing `task31_m5_real10k_pqg8_n64_corpus` and
`_queries` tables (which carry the same DBpedia-style `source real[]`
column), then re-loaded under prefix `m5_diskann_real10k` with profile
`ec_diskann`:

- corpus: `fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv`
  (10000 x 1536d real embeddings, dumped via
  `COPY ... array_to_json(source) ... TO ...`).
- queries: `fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv`
  (200 x 1536d).

Reloptions match Task 29: `graph_degree=32`, `build_list_size=100`,
`alpha=1.2`. Build elapsed: `32.61s`.

### `m5_diskann_real10k_w800` (kernel-stress real)

Same real10k corpus + queries reloaded under prefix
`m5_diskann_real10k_w800` with the additional reloption
`rerank_budget=800`, so each query reranks 800 rows instead of the
default 64. L=800 is needed because `rerank_budget` cannot exceed L
(see `src/am/ec_diskann/scan.rs:185`). Build elapsed: `10.21s`.

## Commands

Generate corpus + queries (synth):

```
ecaz --log-file artifacts/corpus-generate.log corpus generate \
  --output fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv \
  --n 10000 --dim 1536 --seed 42 --kind corpus
ecaz --log-file artifacts/queries-generate.log corpus generate \
  --output fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv \
  --n 200 --dim 1536 --seed 7 --kind queries
```

Dump real10k TSVs from the existing IVF corpus tables (server-side
COPY via `ecaz dev sql`):

```
ecaz dev sql --pg 18 --db postgres --sql \
  "COPY (SELECT id, array_to_json(source) FROM \
   task31_m5_real10k_pqg8_n64_corpus ORDER BY id) \
   TO 'fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv' \
   WITH (FORMAT text, DELIMITER E'\t');"
ecaz dev sql --pg 18 --db postgres --sql \
  "COPY (SELECT id, array_to_json(source) FROM \
   task31_m5_real10k_pqg8_n64_queries ORDER BY id) \
   TO 'fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv' \
   WITH (FORMAT text, DELIMITER E'\t');"
```

Install scalar / NEON binaries and load each prefix:

```
ecaz --log-file artifacts/install-pg18-{scalar,neon}.log \
  dev install ecaz-pg-test --pg 18

ecaz --database postgres --host /Users/peter/.pgrx --port 28818 \
  --log-file artifacts/load-diskann.log \
  corpus load --prefix m5_diskann_synth10k \
    --corpus-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_corpus.tsv \
    --queries-file fixtures/m5_diskann_synth10k/m5_diskann_synth10k_queries.tsv \
    --profile ec_diskann --bits 4 --seed 42 \
    --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2

ecaz ... --log-file artifacts/load-diskann-real10k.log \
  corpus load --prefix m5_diskann_real10k \
    --corpus-file fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv \
    --queries-file fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv \
    --profile ec_diskann --bits 4 --seed 42 \
    --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2

ecaz ... --log-file artifacts/load-diskann-real10k-w800.log \
  corpus load --prefix m5_diskann_real10k_w800 \
    --corpus-file fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv \
    --queries-file fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv \
    --profile ec_diskann --bits 4 --seed 42 \
    --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2 \
    --reloption rerank_budget=800
```

Recall + latency benches per prefix (same shape, swap `--prefix`):

```
ecaz ... bench recall   --prefix <p> --profile ec_diskann --k 10 \
  --sweep <sweep> --force-index \
  --truth-cache-file artifacts/truth_<fixture>_k10.json \
  --log-output artifacts/recall-<arm>-<fixture>-table.log

ecaz ... bench latency  --prefix <p> --profile ec_diskann --k 10 \
  --sweep <sweep> --iterations 200 --concurrency 1 \
  --force-index --sample-backend-memory \
  --log-output artifacts/latency-<arm>-<fixture>-table.log
```

Sweep values:
- `m5_diskann_synth10k`, `m5_diskann_real10k`: `64,200,800` (default rerank_budget=64).
- `m5_diskann_real10k_w800`: `800` only (rerank_budget=800 requires L >= 800).

## Per-fixture results

### synth10k (kernel correctness only)

200-iteration latency, see `latency-{scalar,neon}-table.log`.

| L | scalar mean | NEON mean | scalar p50 | NEON p50 | scalar p95 | NEON p95 | scalar p99 | NEON p99 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.23 ms | 2.27 ms | 2.18 ms | 2.22 ms | 2.51 ms | 2.57 ms | 2.92 ms | 2.76 ms |
| 200 | 2.96 ms | 2.78 ms | 2.86 ms | 2.73 ms | 3.48 ms | 3.08 ms | 4.49 ms | 3.20 ms |
| 800 | 4.12 ms | 3.83 ms | 4.03 ms | 3.77 ms | 4.77 ms | 4.13 ms | 5.21 ms | 4.88 ms |

Recall identical: `0.1650 / 0.2665 / 0.3260` (synthetic high-dim is
near-uniform, so recall is structurally low; do not read this as a
quality result).

### real10k @ default rerank_budget=64

200-iteration latency, see `latency-{scalar,neon}-real-table.log`.

| L | scalar mean | NEON mean | scalar p50 | NEON p50 | scalar p95 | NEON p95 | scalar p99 | NEON p99 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.03 ms | 1.97 ms | 1.98 ms | 1.93 ms | 2.28 ms | 2.16 ms | 2.46 ms | 2.43 ms |
| 200 | 2.21 ms | 2.18 ms | 2.20 ms | 2.15 ms | 2.42 ms | 2.42 ms | 2.54 ms | 2.62 ms |
| 800 | 2.77 ms | 2.73 ms | 2.76 ms | 2.70 ms | 3.09 ms | 2.99 ms | 3.20 ms | 3.16 ms |

Recall identical: `0.9965 / 0.9970 / 0.9975`.

NEON moves p50 by `-0.05 ms` consistently; this is well inside
`~0.20 ms` stddev. The kernel's per-query work at default
`rerank_budget=64` is single-digit microseconds, much smaller than
total query cost.

### real10k_w800 @ rerank_budget=800, L=800 (kernel-stress)

Two passes per arm.

| pass | mean | stddev | min | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| scalar pass 1 (`latency-scalar-real-w800-table.log`) | 17.7 ms | 21.0 ms | 15.4 ms | 16.2 ms | 17.1 ms | 18.8 ms | 313.6 ms |
| scalar pass 2 (`latency-scalar-real-w800-confirm-table.log`) | 17.5 ms | 14.0 ms | 15.6 ms | 16.4 ms | 17.2 ms | 19.0 ms | 215.5 ms |
| NEON pass 1 (`latency-neon-real-w800-table.log`) | 15.1 ms | 0.55 ms | 14.3 ms | 15.0 ms | 15.7 ms | 16.7 ms | 20.6 ms |
| NEON pass 2 (`latency-neon-real-w800-confirm-table.log`) | 15.4 ms | 0.56 ms | 14.7 ms | 15.4 ms | 16.0 ms | 16.7 ms | 20.7 ms |

`mean` and `max` on both scalar passes are inflated by single
`200+ ms` outliers (autovacuum-shaped). The percentile columns are
unaffected and agree across passes.

Pass-averaged percentile deltas:

| metric | scalar avg | NEON avg | delta | rel |
|---|---:|---:|---:|---:|
| min | 15.5 ms | 14.5 ms | `-1.0 ms` | `-6.5%` |
| p50 | 16.3 ms | 15.2 ms | `-1.1 ms` | `-6.7%` |
| p95 | 17.15 ms | 15.85 ms | `-1.3 ms` | `-7.6%` |
| p99 | 18.9 ms | 16.7 ms | `-2.2 ms` | `-11.6%` |

Recall identical: `1.0000 / 1.0000` (`recall-{scalar,neon}-real-w800-table.log`).
NDCG identical: `1.0000 / 1.0000`.

## Artifact list

- install logs: `install-pg18-{scalar,neon}.log`,
  `install-pg18-{scalar,neon}-real.log`,
  `install-pg18-{scalar,neon}-real-w800.log`,
  `install-pg18-scalar-real-w800-confirm.log`
- corpus / queries / load: `corpus-generate.log`, `queries-generate.log`,
  `load-diskann.log`, `load-diskann-real10k.log`,
  `load-diskann-real10k-w800.log`
- synth10k recall + latency: `recall-{scalar,neon}-{table,cli}.log`,
  `latency-{scalar,neon}-{table,cli}.log`,
  `truth_synth10k_k10.json`
- real10k recall + latency: `recall-{scalar,neon}-real-{table,cli}.log`,
  `latency-{scalar,neon}-real-{table,cli}.log`,
  `truth_real10k_k10.json`
- real10k_w800 stress lane:
  `recall-{scalar,neon}-real-w800-{table,cli}.log`,
  `latency-{scalar,neon}-real-w800-{table,cli}.log`,
  `latency-{scalar,neon}-real-w800-confirm-{table,cli}.log`
