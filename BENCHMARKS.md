# Ecaz Benchmark Report

This is an older report template. New packet-backed benchmark rows should
follow [docs/benchmark-reporting-standard.md](docs/benchmark-reporting-standard.md)
and cite packet-local artifacts.

## Environment

| Property | Value |
|---|---|
| Date | |
| CPU | |
| RAM | |
| Storage | |
| OS | |
| Rust | |
| PostgreSQL | |
| Build profile | release, opt-level=3, lto=fat, codegen-units=1, target-cpu=native |
| shared_buffers | |
| work_mem | |

## NFR-001: Query Latency

### Microbenchmarks (criterion, median)

| Function | dim | bits | Median | p95 | Throughput |
|---|---|---|---|---|---|
| `ProdQuantizer::encode` | 1536 | 4 | | | |
| `prepare_ip_query` | 1536 | 4 | | | |
| `score_ip_encoded` | 1536 | 4 | | | |
| `score_ip_codes_lite` | 1536 | 4 | | | |
| `fwht_in_place` | 2048 | - | | | |

### Instruction Counts (iai-callgrind)

| Function | Instructions | L1 misses | Branch misses |
|---|---|---|---|
| `score_ip_encoded` (1536, 4-bit) | | | |
| `fwht_in_place` (2048) | | | |

### Heap Profile (dhat-rs)

| Path | Total bytes | Total blocks | Peak bytes |
|---|---|---|---|
| `encode` (1536, 4-bit, 1K iters) | | | |
| `score_ip_encoded` (1536, 4-bit, 1M iters) | | | |

### HNSW Scan Latency (50K x 1536, 4-bit, top-10)

| Config | ef_search | p50 | p99 | mean |
|---|---|---|---|---|
| m=8 | 40 | | | |
| m=8 | 128 | | | |
| m=16 | 200 | | | |

## NFR-002: Storage Compression

### Per-Vector (1536-dim, 4-bit)

| Component | Bytes |
|---|---|
| Raw fp32 | 6,144 |
| tqvector datum | 783 |
| MSE codes | 576 |
| QJL codes | 192 |
| Gamma | 4 |
| Datum header | 11 |
| Compression ratio | 7.85x |

### Relation Size (50K vectors, 1536-dim, 4-bit)

| Component | Size |
|---|---|
| Element tuples total | |
| Neighbor tuples total | |
| Index relation (pg_relation_size) | |
| Table relation | |
| Total on-disk | |

## NFR-003: Recall Quality

### Dataset

| Property | Value |
|---|---|
| Name | Synthetic (seeded ChaCha8) |
| Corpus size | |
| Dimensions | 1536 |
| Query count | |
| Corpus seed | 42 |
| Query seed | 1000042 |

### Quantizer-Level Recall (sequential scan, no HNSW)

| Metric | Value |
|---|---|
| Recall@10 | |
| Recall@100 | |
| NDCG@10 | |
| MAE (top-10 scores) | |
| Spearman rho | |
| Top-10 set overlap | |

### HNSW Recall Targets

| Config (m, ef) | Recall@10 | Recall@100 | NDCG@10 | Target |
|---|---|---|---|---|
| (8, 128) | | | | >= 89% |
| (8, 200) | | | | >= 93% |
| (16, 200) | | | | >= 97% |

### Ablation: MSE-only vs MSE+QJL

| Estimator | Recall@10 | NDCG@10 |
|---|---|---|
| MSE+QJL | | |
| MSE-only | | |

### Bit-Width Sensitivity (1536-dim)

| bits | Recall@10 | payload bytes | compression |
|---|---|---|---|
| 2 | | 388 | 15.8x |
| 3 | | 580 | 10.6x |
| 4 | | 772 | 7.96x |
| 6 | | 1156 | 5.31x |
| 8 | | 1540 | 3.99x |

### Post-Insert Drift

| % inserted since build | Recall@10 (m=8, ef=128) |
|---|---|
| 0% (fresh) | |
| 5% | |
| 10% | |
| 20% | |
