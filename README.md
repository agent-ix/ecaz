# Ecaz

A PostgreSQL extension written in Rust (pgrx) that provides the canonical
`ecvector(dim)` row type plus HNSW, IVF, and DiskANN index access methods for
approximate nearest neighbor search.

- **`ecvector(dim)`** — canonical exact/raw row type
- **`tqvector`** — explicit TurboQuant artifact/debugging type
- **`<#>` operator** — negative inner product distance for ORDER BY ASC
- **`ec_hnsw` index** — HNSW graph index with per-index storage formats
- **`ec_ivf` index** — IVF posting-list index for measured local tuning and
  high-ingest tradeoffs
- **`ec_diskann` index** — DiskANN/Vamana-style graph index for disk-resident
  experiments
- **`encode_to_ecvector()`** — encode fp32 arrays into the canonical row type

## Quick Start

```bash
cargo install cargo-pgrx@0.17
cargo pgrx init
cargo pgrx install --sudo --release
```

```sql
CREATE EXTENSION ecaz;

CREATE TABLE memories (
    id bigint generated always as identity primary key,
    embedding ecvector(4)
);

-- Encode and store a canonical vector
--   args: float4[] input, codebook_bits (4), rng_seed (42)
INSERT INTO memories (embedding)
VALUES (encode_to_ecvector(ARRAY[1.0, 2.0, 3.0, 4.0]::float4[], 4, 42));

-- Create HNSW index over the canonical row type
CREATE INDEX ON memories
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 8, ef_construction = 64);

-- Query nearest neighbors
SELECT * FROM memories
ORDER BY embedding <#> ARRAY[1.0, 2.0, 3.0, 4.0]::float4[]
LIMIT 10;
```

`tqvector` is not the canonical row type. It is a family-specific TurboQuant
artifact surface for explicit tests, tooling, and debugging. Future persisted
quantized families should add their own family-specific sibling types rather
than overloading `ecvector`.

## Choosing An Index

`ec_hnsw` remains the default general-purpose graph index. It supports storage
formats selected per index with the `storage_format` reloption:

- `turboquant` is the default. Use it for small or medium indexes and for the
  simplest operational path.
- `pq_fastscan` stores a grouped hot path plus a colder rerank payload. Use it
  for latency-critical workloads after measuring it on your corpus.

```sql
-- Default / explicit TurboQuant index
CREATE INDEX ON memories
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (storage_format = 'turboquant', m = 8, ef_construction = 64);

-- PqFastScan index on the same canonical row column
CREATE INDEX ON memories
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (
    storage_format = 'pq_fastscan',
    m = 8,
    ef_construction = 64
);
```

Switching an index from one storage format to the other requires `REINDEX`.
There is no in-place format upgrade.

`ec_ivf` is an opt-in posting-list index. It is useful for comparing sequential
posting-list scan behavior, quantizer variants, and live-insert tradeoffs.

```sql
CREATE INDEX ON memories
USING ec_ivf (embedding ecvector_ip_ops)
WITH (
    nlists = 4,
    nprobe = 2,
    storage_format = 'turboquant',
    rerank = 'heap_f32'
);
```

`ec_diskann` is an opt-in DiskANN/Vamana-style graph index. Local Task 29
measurements established its current build/recall/latency baseline; product
claims still need dedicated benchmark hardware.

```sql
CREATE INDEX ON memories
USING ec_diskann (embedding ecvector_diskann_ip_ops)
WITH (
    graph_degree = 32,
    build_list_size = 100,
    list_size = 128
);
```

## Development

- [Rust](https://rustup.rs/) stable
- [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) `0.17`
- PostgreSQL 17 or 18 development headers

```bash
cargo pgrx init
make fmt
make lint
make lint-pg17
make test
make pg-test
make pg-test-pg17
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
- [pgvector](https://github.com/pgvector/pgvector) (page layout reference)
- [pgrx](https://docs.rs/pgrx/latest/pgrx/)

## License

MIT
