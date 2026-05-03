# Task 31 M5 Quality Rerun After Indexed Ecvector Rerank Decode Specialization

Reviewer: please review this Apple-Silicon-oriented follow-on Task 31 packet.

## Scope

This packet measures committed head `9203de50`, which specializes the IVF
`heap_f32` rerank source load path for the indexed `ecvector` case that Task 31
actually uses.

The hypothesis was narrow and M5-motivated:

- after `886fb369`, the exact rerank dot product on Apple Silicon is already
  NEON-accelerated
- the next per-row cost in the `w1000` quality lane is source decode overhead
- Task 31 rerank rows always fetch the indexed `ecvector` source column, so the
  generic `SourceDatumKind -> FlatFloat4SourceRef` decode path may be avoidable
- specializing that decode path could improve M5 quality latency or explain time

This is a follow-on to `30201`, not a broad IVF cleanup slice.

## Code Checkpoint

- code commit: `9203de50` (`Specialize IVF indexed ecvector rerank decode`)

Focused validation run before measurement:

- `cargo test negative_inner_product_index_internal_matches_scalar_reference --lib`

No broader cargo or pgrx test sweep was run for this packet.

## Commands

Executed:

```text
/Users/peter/.cargo/bin/ecaz --log-file review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/audit.log bench suite audit --config review/30202-task31-m5-quality-ecvector-rerank-decode/task31-m5-ivf-100k-quality.packet.json
/Users/peter/.cargo/bin/ecaz --log-file review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/install-pg18.log dev install ecaz-pg-test --pg 18
/Users/peter/.cargo/bin/ecaz --database postgres --host /Users/peter/.pgrx --port 28818 bench suite run --config review/30202-task31-m5-quality-ecvector-rerank-decode/task31-m5-ivf-100k-quality.packet.json --manifest-output review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/suite-manifest.json --results-output review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/results.jsonl
/Users/peter/.cargo/bin/ecaz --log-file review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/status.log bench suite status --manifest review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/suite-manifest.json
/Users/peter/.cargo/bin/ecaz --log-file review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/report.log bench suite report --manifest review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/suite-manifest.json --results-output review/30202-task31-m5-quality-ecvector-rerank-decode/artifacts/results.jsonl
```

## Results

Surface under test:

- profile: `ec_ivf`
- storage format: `pq_fastscan`
- `pq_group_size=8`
- `nlists=128`
- quality setting under test: `nprobe=96`, `rerank_width=1000`

Quality metrics:

- `nprobe=80`: recall@100 `0.9880`, mean query time `15.02 ms`
- `nprobe=96`: recall@100 `0.9920`, mean query time `12.66 ms`

Latency:

- `nprobe=80`: p50 `10.7 ms`, p95 `11.5 ms`, p99 `12.0 ms`
- `nprobe=96`: p50 `12.1 ms`, p95 `12.9 ms`, p99 `13.4 ms`

Threshold status:

- `quality-candidate-recall100-floor`: pass (`0.992 >= 0.99`)
- `quality-candidate-p50-budget-ms`: pass (`12.1 <= 15.0`)

Representative EXPLAIN/counters for `nprobe=96,rerank_width=1000`:

- index bytes: `20291584` (`19 MB`)
- actual total time: `14.966 ms`
- execution time: `14.989 ms`
- selected lists: `96`
- posting pages read: `1815`
- postings visited: `77760`
- postings scored: `2420`
- postings pruned by bound: `75340`
- candidates inserted: `2420`
- rerank rows: `1000`

## Comparison To `30201`

At the same quality point on prior best M5 checkpoint `886fb369` from `30201`:

| metric | `30201` | `30202` |
|---|---:|---:|
| recall@100 | `0.9920` | `0.9920` |
| recall mean q-time | `12.38 ms` | `12.66 ms` |
| latency p50 | `12.1 ms` | `12.1 ms` |
| latency p95 | `13.0 ms` | `12.9 ms` |
| latency p99 | `13.7 ms` | `13.4 ms` |
| explain execution | `15.309 ms` | `14.989 ms` |
| postings visited | `77760` | `77760` |
| postings scored | `2420` | `2420` |
| postings pruned by bound | `75340` | `75340` |
| rerank rows | `1000` | `1000` |

## Interpretation

This is not a clean enough win to replace `30201`:

- recall stayed fixed
- scan counters stayed fixed
- tail latency and explain improved slightly
- `p50` stayed flat
- recall mean query time moved the wrong way

So this packet should be treated as a mixed result or negative follow-on, not
the new Task 31 M5 baseline.

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
