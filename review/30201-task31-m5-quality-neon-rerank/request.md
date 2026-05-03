# Task 31 M5 Quality Rerun After NEON Exact Rerank Kernel

Reviewer: please review this Apple-Silicon-specific Task 31 checkpoint and its
packet-local quality rerun.

## Scope

This packet measures committed head `886fb369`, which adds an `aarch64` NEON
path for `src/am/ec_hnsw/source.rs::inner_product()`. That exact inner-product
helper is what the IVF `heap_f32` rerank path uses through
`negative_inner_product_index_internal()`.

The hypothesis was M5-specific:

- the Task 31 quality point reranks `1000` heap rows per query
- each rerank row performs an exact `1536`-dimensional dot product
- on Apple Silicon that exact-score kernel was still scalar before this change
- adding a NEON kernel should improve quality-lane latency without changing
  recall or scan counters

This packet intentionally reruns the quality lane first, because that is the
Task 31 point with the highest exact-rerank cost.

## Code Checkpoint

- code commit: `886fb369` (`Add NEON exact rerank inner product`)

Focused validation run before measurement:

- `cargo test negative_inner_product_index_internal_matches_scalar_reference --lib`

No broader cargo or pgrx test sweep was run for this packet; the slice is a
narrow architecture-specific math-kernel change and the required validation
target here is the M5 quality suite rerun.

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30201-task31-m5-quality-neon-rerank/artifacts/audit.log bench suite audit --config review/30201-task31-m5-quality-neon-rerank/task31-m5-ivf-100k-quality.packet.json
/Users/peter/.cargo/bin/ecaz --log-file review/30201-task31-m5-quality-neon-rerank/artifacts/install-pg18.log dev install ecaz-pg-test --pg 18
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30201-task31-m5-quality-neon-rerank/task31-m5-ivf-100k-quality.packet.json --manifest-output review/30201-task31-m5-quality-neon-rerank/artifacts/suite-manifest.json --results-output review/30201-task31-m5-quality-neon-rerank/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30201-task31-m5-quality-neon-rerank/artifacts/status.log bench suite status --manifest review/30201-task31-m5-quality-neon-rerank/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30201-task31-m5-quality-neon-rerank/artifacts/report.log bench suite report --manifest review/30201-task31-m5-quality-neon-rerank/artifacts/suite-manifest.json --results-output review/30201-task31-m5-quality-neon-rerank/artifacts/results.jsonl
```

## Results

Surface under test:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- quality setting under test: `nprobe=96`, `rerank_width=1000`

Quality metrics:

- `nprobe=80`: recall@100 `0.9880`, mean query time `14.10 ms`
- `nprobe=96`: recall@100 `0.9920`, mean query time `12.38 ms`

Latency:

- `nprobe=80`: p50 `10.7 ms`, p95 `11.6 ms`, p99 `12.7 ms`
- `nprobe=96`: p50 `12.1 ms`, p95 `13.0 ms`, p99 `13.7 ms`

Threshold status:

- `quality-candidate-recall100-floor`: pass (`0.992 >= 0.99`)
- `quality-candidate-p50-budget-ms`: pass (`12.1 <= 15.0`)

Representative EXPLAIN/counters for `nprobe=96,rerank_width=1000`:

- index bytes: `20291584` (`19 MB`)
- actual total time: `15.280 ms`
- execution time: `15.309 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `2420`
- postings pruned by bound: `75340`
- candidates inserted: `2420`
- rerank rows: `1000`

## Comparison To `30195`

At the same quality point on prior best checkpoint `c1a761fd` from `30195`:

| metric | `30195` | `30201` |
|---|---:|---:|
| recall@100 | `0.9920` | `0.9920` |
| recall mean q-time | `13.13 ms` | `12.38 ms` |
| latency p50 | `12.8 ms` | `12.1 ms` |
| latency p95 | `13.5 ms` | `13.0 ms` |
| latency p99 | `13.9 ms` | `13.7 ms` |
| explain execution | `15.941 ms` | `15.309 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `2420` | `2420` |
| postings pruned by bound | `75340` | `75340` |
| rerank rows | `1000` | `1000` |

## Interpretation

This looks like a real M5-specific win rather than generic scan noise:

- recall stayed fixed
- all scan and rerank counters stayed fixed
- the quality lane improved across mean q-time, p50, p95, p99, and explain
- the only intended behavior change was the Apple-Silicon exact-score kernel

That makes the NEON exact rerank math the current highest-value Apple-Silicon
checkpoint tested so far for Task 31 quality.

## Artifacts

- `artifacts/audit.log`
- `artifacts/install-pg18.log`
- `artifacts/suite-manifest.json`
- `artifacts/results.jsonl`
- `artifacts/status.log`
- `artifacts/report.log`
- `artifacts/recall100_real100k_pqg8_n128_w1000_p80_96.log`
- `artifacts/latency_real100k_pqg8_n128_w1000_p80_96.log`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.sql`
- `artifacts/explain_real100k_pqg8_n128_p96_w1000.log`
- `artifacts/truth_real100k_n128_k100.json`
- `artifacts/manifest.md`
