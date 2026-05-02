<img width="1254" height="1254" alt="2e7cab9b-880a-45ba-9d0b-ce2d78271265" src="https://github.com/user-attachments/assets/0fceb7fa-9646-49ae-b46a-e8bdeac2caec" />

Ecaz is a PostgreSQL extension written in Rust with a focus on performant,
highly scalable vector storage and retrieval. It aims to support a broad range of quantization and
index options rather than a single fixed architecture. Current index families
include HNSW (`ec_hnsw`), IVF (`ec_ivf`), and DiskANN/Vamana-style graphs
(`ec_diskann`).

## This software was written 100% by AI

Ecaz is an Agentic Engineering experiment: an attempt to develop a complex
database system written solely by AI. A human worked with AI to design the
architecture and navigate the many design decisions, but 100% of the code was 
written by GPT >=5.4 and Claude Opus >=4.6.

**The ethos is to pursue quality, testing,
and benchmarking rigorously, but the project should not yet be considered
production-ready.**

Having achieved the initial goal of support for well-known index
families, the project now aims to build proof-of-concept implementations for
frontier vector database research.

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

`ec_diskann` is an opt-in DiskANN/Vamana-style graph index. It currently
expects unit-normalized source vectors. Local Task 29 measurements established
its current build/recall/latency baseline; product claims still need dedicated
benchmark hardware.

```sql
CREATE TABLE unit_memories (
    id bigint generated always as identity primary key,
    embedding ecvector(4)
);

INSERT INTO unit_memories (embedding)
VALUES
    (encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0]::float4[], 4, 42)),
    (encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0]::float4[], 4, 42));

CREATE INDEX ON unit_memories
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

Measured local results on 1536-dimensional OpenAI embeddings
([DBpedia corpus](docs/recall-methodology.md)):

| Surface | Current local result |
| --- | --- |
| Compression | 7.85x vs fp32 (783 bytes per 1536-dim vector) |
| HNSW recall@10 | 97.1% - 97.5% on 10K; 92.6% - 95.2% on 50K |
| IVF 100K selected point | Recall@10 0.9920, p50 173.4 ms, 19,791,872 B index |
| DiskANN real-10K selected point | Recall@10 0.9965 - 0.9975, mean 7.80 - 9.34 ms, 4,939,776 B index |

These are local engineering results, not product benchmark claims. See
[Benchmarks](docs/benchmarks.md) for full results, source packets, and
methodology.

The supported operator workflow uses the `ecaz` CLI:

```bash
cargo install --path crates/ecaz-cli
ecaz corpus prepare --profile ec_hnsw_real_10k --parquet /path/to/parquet --output-dir /path/to/staged
ecaz corpus load --prefix ec_hnsw_real_10k --corpus-file /path/to/staged/ec_hnsw_real_10k_corpus.tsv --queries-file /path/to/staged/ec_hnsw_real_10k_queries.tsv --profile ec_hnsw
ecaz bench recall --prefix ec_hnsw_real_10k --profile ec_hnsw
```

Use `--log-file review/<topic>/artifacts/<run>.log` when producing review
packet evidence.

## Documentation

| Document | Description |
| --- | --- |
| [Getting Started](docs/getting-started.md) | Prerequisites, installation, first query |
| [Usage Guide](docs/usage.md) | Encoding parameters, index tuning, query patterns |
| [Benchmarks](docs/benchmarks.md) | Measured performance results and methodology |
| [Operator CLI](crates/ecaz-cli/README.md) | `ecaz` corpus, benchmark, compare, stress, and dev command surface |
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
