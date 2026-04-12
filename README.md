# tqvector

A PostgreSQL extension written in Rust (pgrx) that provides the `tqvector` data type and `tqhnsw` index access method for approximate nearest neighbor search over TurboQuant-compressed vectors.

- **`tqvector` type** — TurboQuant-compressed vector codes (~8x smaller than fp32)
- **`<#>` operator** — negative inner product distance for ORDER BY ASC
- **`tqhnsw` index** — HNSW graph index over compressed codes
- **`encode_to_tqvector()`** — compress fp32 arrays to tqvector in SQL

## Quick Start

```bash
cargo install cargo-pgrx@0.17
cargo pgrx init
cargo pgrx install --sudo --release
```

```sql
CREATE EXTENSION tqvector;

-- Encode and store a vector
--   args: float4[] input, codebook_bits (4), rng_seed (42)
INSERT INTO memories (tq_code)
VALUES (encode_to_tqvector(ARRAY[1.0, 2.0, ...]::float4[], 4, 42));

-- Create HNSW index
CREATE INDEX ON memories USING tqhnsw (tq_code) WITH (m=8, ef_construction=64);

-- Query nearest neighbors
SELECT * FROM memories
ORDER BY tq_code <#> encode_to_tqvector($query::float4[], 4, 42)
LIMIT 10;
```

## Performance

Measured on 1536-dimensional OpenAI embeddings ([DBpedia corpus](docs/recall-methodology.md)):

| Metric | Value |
| --- | --- |
| Compression | 7.85x vs fp32 (783 bytes per 1536-dim vector) |
| Recall@10 (10K, m=8) | 97.1% – 97.5% |
| Recall@10 (50K, m=8) | 92.6% – 95.2% |
| Latency target | p50 < 5ms, p99 < 15ms (top-10 on 50K) |

See [Benchmarks](docs/benchmarks.md) for full results and methodology.

## Documentation

| Document | Description |
| --- | --- |
| [Getting Started](docs/getting-started.md) | Prerequisites, installation, first query |
| [Usage Guide](docs/usage.md) | Encoding parameters, index tuning, query patterns |
| [Benchmarks](docs/benchmarks.md) | Measured performance results and methodology |
| [Architecture](docs/architecture.md) | Compression pipeline, index layout, page format |
| [PG18 Features](docs/pg18.md) | ReadStream, EXPLAIN hooks, AM callbacks |
| [Contributing](docs/contributing.md) | Makefile targets, CI, testing, fuzzing |

## Project

| Resource | Description |
| --- | --- |
| [Specification](spec/spec.md) | Master requirements specification |
| [Implementation Plan](plan/plan.md) | Task board, sequencing, status |
| [ADRs](spec/adr/) | Architecture decision records |
| [Reviews](review/) | Review packets and feedback ([workflow](AGENTS.md)) |

## References

- [TurboQuant paper (arXiv:2504.19874)](https://arxiv.org/abs/2504.19874)
- [hnsw_rs crate](https://crates.io/crates/hnsw_rs)
- [pgvector](https://github.com/pgvector/pgvector) (page layout reference)
- [pgrx](https://docs.rs/pgrx/latest/pgrx/)

## License

MIT
