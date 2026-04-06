# tqvector

A PostgreSQL extension written in Rust (pgrx) that registers the `tqvector` data type and `tqhnsw` index access method for approximate nearest neighbor search over TurboQuant-compressed vectors.

## What

- **`tqvector` type** — stores TurboQuant-compressed vector codes (8-10x smaller than fp32)
- **`<#>` operator** — negative inner product for ORDER BY ASC (highest similarity first)
- **`tqhnsw` index** — HNSW graph index over compressed codes, modeled on pgvector's page layout
- **`encode_to_tqvector()`** — compress fp32 arrays to tqvector in SQL

## Why

Existing options don't work for us:
- pgvecto.rs — deprecated
- VectorChord — AGPL/ELv2 licensing
- pgvector — MIT but stores fp32 (no compression, 8x larger)

This extension is MIT licensed, implements its own data-oblivious TurboQuant quantizer core in-tree, and uses the `hnsw_rs` crate for graph construction.

## Usage

```sql
CREATE EXTENSION tqvector;

-- Encode and store
INSERT INTO memories (tq_code)
VALUES (encode_to_tqvector(ARRAY[1.0, 2.0, ...]::float4[], 4, 42));

-- Create HNSW index
CREATE INDEX ON memories USING tqhnsw (tq_code) WITH (m=8, ef_construction=64);

-- Query nearest neighbors
SELECT * FROM memories
ORDER BY tq_code <#> encode_to_tqvector($query::float4[], 4, 42)
LIMIT 10;
```

## Prerequisites

- [Rust](https://rustup.rs/) stable
- [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) matching the `pgrx` version in `Cargo.toml`:
  ```bash
  cargo install cargo-pgrx --version "^0.17"
  ```
- System packages (Ubuntu/Debian):
  ```bash
  sudo apt-get install -y postgresql-server-dev-all postgresql-common \
    build-essential pkg-config libssl-dev libreadline-dev bison flex
  ```

## Getting Started

```bash
cargo pgrx init          # builds local Postgres instances for pg14–pg18
make fmt                 # format code
make lint                # clippy (deny warnings)
make test                # unit tests
make pg-test             # pgrx integration tests (pg18 by default)
make install             # install into local PG
```

To run integration tests against a specific version:

```bash
cargo pgrx test pg18
```

## Architecture

See `spec/spec.md` for the full technical specification and `~/dev/agent-memory-context.md` for the system-level architecture.

## References

- [TurboQuant paper (arXiv:2504.19874)](https://arxiv.org/abs/2504.19874)
- [hnsw_rs crate](https://crates.io/crates/hnsw_rs)
- [pgvector source](https://github.com/pgvector/pgvector) (page layout reference)
- [pgrx framework](https://docs.rs/pgrx/latest/pgrx/)

## License

MIT
