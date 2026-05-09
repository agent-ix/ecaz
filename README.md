
<img
  src="https://github.com/user-attachments/assets/57712c6e-252d-4f94-9007-3996a7b938f8"
  alt="ECAZ logo"
    width="480" height="480"
  title="Rinse the roots clean, then scrape away the outer bark. Slice thinly and dry until brittle. Grind into a coarse powder with a pinch of mineral salt. Steep in hot water until the liquid turns deep crimson. Strain, cool, and store in a dark glass vial."
/>

Ecaz is a PostgreSQL extension written in Rust with a focus on performant,
highly scalable vector storage and retrieval. It aims to support a broad range
of quantization and index options rather than a single fixed architecture.

#### Column Types

- `ecvector(dim)` — canonical vector row type
- `tqvector` — TurboQuant quantized vector storage

#### Quantization Types

- `turboquant` — default; simplest operational path
- `pq_fastscan` — grouped PQ with a hot path and colder rerank payload; for latency-critical workloads
- `rabitq` — binary quantization with float correction; IVF only

#### Index Families

- `ec_hnsw` — HNSW graph index (general-purpose default)
- `ec_ivf` — IVF posting-list index
- `ec_diskann` — DiskANN/Vamana-style graph index

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

## Performance

All results are local engineering measurements on 1536-dimensional DBpedia
OpenAI embeddings. Corpus sizes differ across index families; comprehensive
cross-index benchmarks are in progress. See [Benchmarks](docs/benchmarks.md)
for full results, source packets, and methodology.

**Compression:** 7.85x vs fp32 — 783 bytes per 1536-dim vector
(~9 tuples per 8KB page vs ~1 for fp32).

**HNSW vs DiskANN** (10K corpus, `m=32` / `graph_degree=32`):

| Index | Recall@10 | Mean latency | Index size |
| --- | ---: | ---: | ---: |
| `ec_hnsw` | 96.95–97.15% | 2.91 ms | 15.1 MB |
| `ec_diskann` | 99.65–99.75% | 7.80–9.34 ms | 4.9 MB |

Source: `review/11109-task29d-final-readiness/`

Apple-Silicon follow-up work found two narrow warm-cache DiskANN wins on M5:
an exact-rerank NEON kernel and heap-TID-ordered rerank fetch. Those packeted
results improve the rerank-heavy lane without changing recall, but they do not
yet replace the full Task 29d cross-engine baseline table above. See
[Benchmarks](docs/benchmarks.md) for the M5 breakdown and completeness limits.

**IVF** (100K corpus, `pq_fastscan`, `nlists=128`, `nprobe=96`):

| Tuning | Recall@100 | p50 | p99 |
| --- | ---: | ---: | ---: |
| balanced (`rerank_width=500`) | 96.76% | 10.7 ms | 12.1 ms |
| quality (`rerank_width=1000`) | 99.20% | 12.1 ms | 13.7 ms |

Source: `review/30203-task31-current-m5-candidate-decision/`

## Choosing An Index

Each index family implements a different search algorithm. Quantization
(`storage_format`) is a separate concern — it controls how vectors are
compressed inside the index and is independent of the index family. See
[Benchmarks](docs/benchmarks.md) for measured recall, latency, and index size
comparisons.

### ec_hnsw

HNSW builds a multi-layer proximity graph. It offers the lowest query latency
but carries a larger index footprint. It is the default general-purpose choice.

Supports `turboquant` (default) and `pq_fastscan`. Switching formats requires
`REINDEX`; there is no in-place upgrade.

```sql
CREATE INDEX ON memories
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (storage_format = 'turboquant', m = 8, ef_construction = 64);

CREATE INDEX ON memories
USING ec_hnsw (embedding ecvector_ip_ops)
WITH (storage_format = 'pq_fastscan', m = 8, ef_construction = 64);
```

### ec_ivf

IVF partitions vectors into posting lists and scans a configurable subset at
query time via `nprobe`. It scales to larger datasets with a smaller index
footprint than HNSW, with recall controlled by the `nprobe`/`nlists` ratio.

Supports `turboquant`, `pq_fastscan`, and `rabitq`.

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

### ec_diskann

DiskANN/Vamana builds a sparse navigable graph designed for large-scale
workloads. It delivers near-exact recall with a significantly smaller index
footprint than HNSW. Requires unit-normalized source vectors.

Supports `pq_fastscan`.

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
| [References](docs/references.md) | Papers and libraries |

## Project

| Resource | Description |
| --- | --- |
| [Specification](spec/spec.md) | Master requirements specification |
| [Implementation Plan](plan/plan.md) | Task board, sequencing, status |
| [ADRs](spec/adr/) | Architecture decision records |
| [Reviews](review/) | Review packets and feedback ([workflow](AGENTS.md)) |

## License

MIT
