# Cloud REAL baselines — DBpedia 10k + 50k, ec_ivf @ m8g.large (PG18)

## Purpose

Pre-IVF/RaBitQ-optimization baselines on real Qdrant DBpedia OpenAI
text-embedding-3-large data (1536-dim). Replaces the
[synthetic baselines](../cloud-10k-baselines/) — those vectors had no
cluster structure, recall ≈ fraction of cells probed; not
representative of production behavior.

## Environment

| Property | Value |
|---|---|
| Region | us-west-2 (AZ us-west-2a) |
| DB instance | m8g.large (Graviton 4, 2 vCPU / 8 GB) |
| Loader instance | c8g.medium (Graviton 4) — **bypassed**, all work on DB host |
| EBS | gp3, 50 GB, 3000 IOPS, 125 MiB/s |
| OS | Amazon Linux 2023 (kernel 6.1, aarch64) |
| PostgreSQL | 18.3 |
| ecaz | 0.1.1 |
| Date (UTC) | 2026-05-16 |

## Datasets

`Qdrant/dbpedia-entities-openai3-embedding-3-large-1536-1M` (26 parquet
shards, 1M total rows). Subsets sliced by `ecaz corpus prepare` using
built-in `SubsetProfile`s (sorted-id prefix, deterministic):

| Prefix | Corpus rows | Query rows | Source |
|---|---|---|---|
| `ec_hnsw_real_10k` | 10,000 | 200 | DBpedia 1M parquet |
| `ec_hnsw_real_50k` | 50,000 | 1,000 | DBpedia 1M parquet |

## Indexes

`ec_ivf` at default reloptions:

- `ec_hnsw_real_10k_idx` built in 11.04 s
- `ec_hnsw_real_50k_idx` built in 33.06 s

## Bench parameters

`ecaz bench latency` / `ecaz bench recall`:

| Parameter | Value |
|---|---|
| `k` | 10 |
| `iterations` | 200 (latency) |
| `concurrency` | 1 |
| `sweep` | profile default `nprobe = [8, 16, 24, 32, 48, 64]` |

## Results — ec_hnsw_real_10k (real DBpedia 10k)

### Latency (200 iters, k=10, concurrency=1)

| nprobe | mean | p50 | p95 | p99 |
|---|---|---|---|---|
| 8 | 4.05 ms | 3.17 ms | 8.53 ms | 14.8 ms |
| 16 | 2.89 ms | 2.92 ms | 3.35 ms | 5.92 ms |
| 24 | 3.93 ms | 3.98 ms | 4.25 ms | 4.58 ms |
| 32 | 4.91 ms | 4.92 ms | 5.45 ms | 5.68 ms |
| 48 | 7.00 ms | 7.01 ms | 7.51 ms | 7.77 ms |
| 64 | 9.15 ms | 9.11 ms | 9.78 ms | 10.2 ms |

(Note: nprobe=8 row has elevated stddev; cold-cache warmup artifact —
mean settles by nprobe=16.)

### Recall@10

| nprobe | recall@10 | ndcg@10 | mean q-time |
|---|---|---|---|
| 8 | **0.9690** | 0.9994 | 1.70 ms |
| 16 | 0.9730 | 0.9997 | 2.82 ms |
| 24 | 0.9740 | 0.9998 | 3.95 ms |
| 32 | 0.9745 | 0.9998 | 4.88 ms |
| 48 | 0.9745 | 0.9998 | 6.99 ms |
| 64 | 0.9745 | 0.9998 | 9.11 ms |

Recall saturates at ~97% by nprobe=24. The remaining 3% is the quantization
recall ceiling (4-bit codes lose information that no nprobe can recover).

## Results — ec_hnsw_real_50k (real DBpedia 50k)

### Latency (200 iters, k=10, concurrency=1)

| nprobe | mean | p50 | p95 | p99 |
|---|---|---|---|---|
| 8 | 3.36 ms | 3.30 ms | 4.08 ms | 5.36 ms |
| 16 | 5.56 ms | 5.51 ms | 6.37 ms | 7.24 ms |
| 24 | 7.99 ms | 8.01 ms | 8.98 ms | 9.16 ms |
| 32 | 10.3 ms | 10.3 ms | 11.4 ms | 11.9 ms |
| 48 | 15.1 ms | 15.2 ms | 16.4 ms | 17.2 ms |
| 64 | 19.7 ms | 19.6 ms | 21.2 ms | 21.9 ms |

### Recall@10

| nprobe | recall@10 | ndcg@10 | mean q-time |
|---|---|---|---|
| 8 | **0.8290** | 0.9886 | 3.52 ms |
| 16 | 0.8863 | 0.9941 | 5.61 ms |
| 24 | 0.9107 | 0.9961 | 8.16 ms |
| 32 | 0.9236 | 0.9974 | 10.60 ms |
| 48 | 0.9364 | 0.9985 | 16.14 ms |
| 64 | 0.9414 | 0.9989 | 21.10 ms |

50k recall climbs more slowly than 10k (more cells → more probing needed
for the same coverage). Reaches 94% by nprobe=64.

## Synth-vs-real recall comparison @ nprobe=8

| Corpus | Synth (uniform) | Real (DBpedia) |
|---|---|---|
| 10k @ nprobe=8 | 14.4% | **96.9%** |
| 50k @ nprobe=8 | 7.0% | **82.9%** |

Real data has the cluster structure IVF was designed for; synth uniform
data has none.

## Preserved artifacts

- **EBS snapshot**: `snap-054feaffc50ecf1c9` — full PGDATA with
  both real corpora + queries + indexes. Restore via
  `ecaz cloud up --profile 10k --from-snapshot <id>` to skip the
  ~10 min ecaz build + ~3 min parquet fetch + ~1 min subset prepare
  + ~1 min load+index build.
- **Raw logs**: `artifacts/m8g.large/ec_ivf/real{10k,50k}-{latency,recall}.log`

## How to re-run after IVF / RaBitQ optimization

```bash
ecaz cloud up --profile 10k --from-snapshot <snap-id>
ecaz cloud install --profile 10k --git-ref <new-sha>   # rebuild ecaz only

# ssm to DB host:
ecaz bench latency --prefix ec_hnsw_real_10k --profile ec_ivf --k 10 --iterations 200 --concurrency 1
ecaz bench recall  --prefix ec_hnsw_real_10k --profile ec_ivf --k 10
ecaz bench latency --prefix ec_hnsw_real_50k --profile ec_ivf --k 10 --iterations 200 --concurrency 1
ecaz bench recall  --prefix ec_hnsw_real_50k --profile ec_ivf --k 10

ecaz cloud snapshot --profile 10k --description "post-opt real baselines"
ecaz cloud down --profile 10k --yes
```
