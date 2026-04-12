# Benchmarks

## Recall

Measured on the [DBpedia OpenAI embeddings corpus](recall-methodology.md) (1536-dim, `text-embedding-3-large`).

### 10K Corpus (m=8)

| ef_search | Recall@10 |
| --- | --- |
| sweep | 97.1% – 97.5% |

### 50K Corpus (m=8)

| ef_search | Recall@10 |
| --- | --- |
| sweep | 92.6% – 95.2% |

### Targets (NFR-003)

| Configuration | Target |
| --- | --- |
| m=8, ef=128 | >= 89% |
| m=8, ef=200 | >= 93% |
| m=16, ef=200 | >= 97% |

## Storage

| Metric | Value |
| --- | --- |
| Raw fp32 (1536-dim) | 6,144 bytes |
| tqvector (4-bit) | 783 bytes |
| Compression ratio | 7.85x |
| Tuples per 8KB page | ~9 (vs ~1 for pgvector) |

## Latency

Targets (NFR-001) for top-10 query on 50K vectors:

| Percentile | Target |
| --- | --- |
| p50 | < 5ms |
| p99 | < 15ms |

## Running Benchmarks

### Criterion microbenchmarks

```bash
make bench            # all criterion benchmarks
make bench-quant_score  # specific benchmark
```

### Instruction-count benchmarks (requires valgrind)

```bash
make bench-iai
```

### SQL benchmarks (requires running PostgreSQL with extension installed)

```bash
make bench-sql-latency
make bench-storage
make bench-recall-sql
```

### Heap profiling

```bash
make dhat-encode
make dhat-score
```

## Methodology

See [Recall Methodology](recall-methodology.md) for the dataset contract, corpus selection rules, and reproduction instructions.
