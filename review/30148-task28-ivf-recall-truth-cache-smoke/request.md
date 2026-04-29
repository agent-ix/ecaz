# Task 28 IVF Recall Truth Cache Smoke

## Scope

Validate the new `ecaz bench recall --truth-cache-dir` path against an existing
PG18 IVF surface before using it for larger A9/A10 recall sweeps.

Fixture:

- prefix: `task28_ivf_pqg10k_g8_n128`
- profile: `ec_ivf`
- `k=10`
- `queries-limit=3`
- `nprobe=8`
- `rerank_width=500`
- cache state: warm local PG18; no OS or PostgreSQL cache drop

## Result

The first run wrote an exact-truth cache file:

```text
truth-v1-rows10000-queries3-dim1536-k10-eb27c241304e37df.json
```

The second run loaded the same cache file and produced the same recall/NDCG:

| run | recall@10 | NDCG@10 | mean q-time |
|---|---:|---:|---:|
| cache miss/write | 0.8667 | 0.9934 | 85.95 ms |
| cache hit/read | 0.8667 | 0.9934 | 59.98 ms |

This is a harness smoke only. Do not use the three-query latency delta as a
performance claim.

## Validation

- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --queries-limit 3 --sweep 8 --rerank-width 500 --force-index --truth-cache-dir review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/truth-cache --log-output review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/recall_cache_miss.log`
- `cargo run -p ecaz-cli -- --database postgres --host /home/peter/.pgrx --port 28818 bench recall --prefix task28_ivf_pqg10k_g8_n128 --profile ec_ivf --k 10 --queries-limit 3 --sweep 8 --rerank-width 500 --force-index --truth-cache-dir review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/truth-cache --log-output review/30148-task28-ivf-recall-truth-cache-smoke/artifacts/recall_cache_hit.log`

## Artifacts

- `artifacts/recall_cache_miss.log`
- `artifacts/recall_cache_hit.log`
- `artifacts/truth-cache/truth-v1-rows10000-queries3-dim1536-k10-eb27c241304e37df.json`
- `artifacts/manifest.md`
