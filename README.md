
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

## Quick Start

The most repeatable local path is a pgrx-managed PostgreSQL 18 instance.
This avoids depending on whichever `pg_config` happens to be first on `PATH`.

```bash
cargo install cargo-pgrx@0.17
cargo pgrx init --pg18 download
cargo pgrx run --release pg18
```

`cargo pgrx run` builds the extension, installs it into the managed PG18
cluster, starts PostgreSQL if needed, and opens `psql`.

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
SELECT id FROM memories
ORDER BY embedding <#> ARRAY[1.0, 2.0, 3.0, 4.0]::float4[]
LIMIT 10;
```

See [Build From Source](docs/build-from-source.md) for the full repeatable
setup path, including native prerequisites, existing-PostgreSQL installs,
operator CLI setup, PG17 compatibility, and validation commands.

## Compatibility

| Area | Status |
| --- | --- |
| PostgreSQL | PG18 primary target; PG17 compatibility target |
| pgrx | `cargo-pgrx` 0.17 |
| Rust | Stable toolchain |
| Linux | Active development and test platform on x86_64 |
| macOS | Active development and benchmark platform on Apple Silicon, including Apple M5 IVF and DiskANN tuning lanes |
| CPU target | Local builds use `target-cpu=native`; build release artifacts on the same CPU family that will run them |

## Build From Source

Ecaz targets PG18 by default, with PG17 kept as a compatibility build. A
complete source setup has five parts:

1. Install Rust stable, native build tools, and PostgreSQL build dependencies.
2. Install the matching pgrx toolchain: `cargo install cargo-pgrx@0.17`.
3. Initialize pgrx for PG18: `cargo pgrx init --pg18 download`.
4. Build and install into a pgrx-managed PG18: `cargo pgrx run --release pg18`.
5. Install the operator CLI for repeatable local SQL, corpus, and benchmark
   commands: `cargo install --path crates/ecaz-cli`.

For an already-installed PostgreSQL server, install with an explicit
`pg_config` instead:

```bash
cargo pgrx install --sudo --release --pg-config /path/to/pg_config
```

The detailed guide is [docs/build-from-source.md](docs/build-from-source.md).

## Performance

All results are local engineering measurements on 1536-dimensional DBpedia
OpenAI embeddings. Corpus sizes differ across index families; comprehensive
cross-index benchmarks are in progress. See [Benchmarks](docs/benchmarks.md)
for full results, source packets, and methodology.

### Compression And Storage Format

For 1536-dimensional vectors:

| Representation | Bytes per vector | Relative size | 8KB intuition |
| --- | ---: | ---: | --- |
| Raw fp32 | 6,144 B | 1.00x | about 1 source vector |
| TurboQuant 4-bit artifact | 783 B | 7.85x smaller | about 9 tuples with ordinary tuple overhead |
| PQ-FastScan g8 search code | 96 B | 64.0x smaller | about 85 code payloads before AM overhead |
| RaBitQ 4-bit IVF code | 780 B | 7.88x smaller | about 10 code payloads before AM overhead |

The PQ-FastScan and RaBitQ rows are per-vector code payloads from the current
implementation. Full index footprint also includes access-method tuple/list or
graph overhead, codebooks, and any rerank sidecar data.

Apple-Silicon follow-up work found two narrow warm-cache DiskANN wins on M5:
an exact-rerank NEON kernel and heap-TID-ordered rerank fetch. Those packeted
results improve the rerank-heavy lane without changing recall, but they do not
yet replace the full Task 29d cross-engine baseline table above. See
[Benchmarks](docs/benchmarks.md) for the M5 benchmark inventory and packet
sources.

Index footprint depends on both access method and storage format. On the local
IVF 10K/25K matched-width lane (`nlists=64`, `nprobe=48`,
`rerank='heap_f32'`, `rerank_width=750`), the storage-format tradeoff was:

| Corpus | IVF storage format | Recall@100 | p50 | Index size |
| --- | --- | ---: | ---: | ---: |
| 10K | `turboquant` | 99.66% | 130.6 ms | 9.6 MB |
| 10K | `pq_fastscan`, `pq_group_size=8` | 93.60% | 77.3 ms | 2.5 MB |
| 10K | `rabitq` | 99.30% | 344.2 ms | 9.6 MB |
| 25K | `turboquant` | 99.29% | 284.5 ms | 23.3 MB |
| 25K | `pq_fastscan`, `pq_group_size=8` | 92.56% | 116.8 ms | 5.3 MB |
| 25K | `rabitq` | 99.15% | 775.7 ms | 23.5 MB |

Source: `review/30145-task28-ivf-a10-current-closure/`

For high-dimensional 100K IVF surfaces, the measured recommendation is explicit
`storage_format = 'pq_fastscan', pq_group_size = 8`; smaller 10K/25K workloads
may prefer the higher recall@100 behavior of `turboquant`.

### Index Family Snapshot

These rows are not a single controlled cross-index benchmark; they are the
current local engineering anchors for each access method.

| Access method | Corpus / platform | Configuration | Recall | Latency | Index size |
| --- | --- | --- | ---: | ---: | ---: |
| `ec_hnsw` | 10K local PG18 | `m=32`, `ef_construction=100` | 96.95-97.15% @10 | mean 2.91 ms | 15.1 MB |
| `ec_ivf` | 100K local PG18 | `pq_fastscan` g8, `nlists=128`, `nprobe=48`, `rerank_width=500` | 99.20% @10 / 95.52% @100 | p50 169.3 ms / p99 194.4 ms | 19.8 MB |
| `ec_ivf` | 100K Apple M5, balanced | `pq_fastscan` g8, `nlists=128`, `nprobe=96`, `rerank_width=500` | 96.76% @100 | p50 10.7 ms / p99 12.1 ms | same surface |
| `ec_ivf` | 100K Apple M5, quality | `pq_fastscan` g8, `nlists=128`, `nprobe=96`, `rerank_width=1000` | 99.20% @100 | p50 12.1 ms / p99 13.7 ms | same surface |
| `ec_diskann` | 10K local PG18 | `graph_degree=32`, `build_list_size=100`, `alpha=1.2` | 99.65-99.75% @10 | mean 7.80-9.34 ms | 4.9 MB |

Sources: `review/11109-task29d-final-readiness/`,
`review/30145-task28-ivf-a10-current-closure/`,
`review/30203-task31-current-m5-candidate-decision/`

## Choosing An Index

Each index family implements a different search algorithm. Quantization
(`storage_format`) is a separate concern — it controls how vectors are
compressed inside the index and is independent of the index family. See
[Usage Guide](docs/usage.md) for full SQL examples and [Benchmarks](docs/benchmarks.md)
for measured recall, latency, and index size comparisons.

| Access method | Best fit | Storage formats | Notes |
| --- | --- | --- | --- |
| `ec_hnsw` | General-purpose ANN graph search | `turboquant`, `pq_fastscan` | Lowest local latency, larger index footprint |
| `ec_ivf` | Posting-list experiments and high-ingest tradeoffs | `turboquant`, `pq_fastscan`, `rabitq` | Recall controlled by `nprobe`/`nlists`; Apple M5 tuning is active |
| `ec_diskann` | DiskANN/Vamana research and compact graph indexes | `pq_fastscan` | Requires unit-normalized source vectors; Apple M5 tuning is active |

Changing an index storage format requires `REINDEX`; there is no in-place
format upgrade.

## Development

- [Rust](https://rustup.rs/) stable
- [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) `0.17`
- Native PostgreSQL build dependencies, or PostgreSQL 17/18 development
  headers if using an existing server

```bash
cargo pgrx init --pg18 download
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
| [Build From Source](docs/build-from-source.md) | Full repeatable local build and setup path |
| [Usage Guide](docs/usage.md) | Encoding parameters, index tuning, query patterns |
| [Benchmarks](docs/benchmarks.md) | Measured performance results and methodology |
| [Benchmark Index](docs/benchmark-index.md) | Packet directory for benchmark lanes and source artifacts |
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
