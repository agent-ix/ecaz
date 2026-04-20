# Getting Started

## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) 0.17: `cargo install cargo-pgrx@0.17`
- PostgreSQL 17 or 18 development headers

## Setup

Initialize a local PostgreSQL instance for development:

```bash
cargo pgrx init
```

Build and install the extension:

```bash
cargo pgrx install --sudo --release
```

## First Query

Connect to your local PostgreSQL and try:

```sql
CREATE EXTENSION ecaz;

-- Create a table with a tqvector column
CREATE TABLE items (
    id serial PRIMARY KEY,
    embedding tqvector
);

-- Encode and insert a vector
--   encode_to_tqvector(input, codebook_bits, rng_seed)
--     input:          float4[] — the raw embedding
--     codebook_bits:  integer  — quantization depth (e.g. 4)
--     rng_seed:       bigint   — deterministic seed for the random rotation
INSERT INTO items (embedding)
VALUES (encode_to_tqvector(ARRAY[1.0, 2.0, 3.0, ...]::float4[], 4, 42));

-- Create an HNSW index
CREATE INDEX ON items USING ec_hnsw (embedding) WITH (m=8, ef_construction=64);

-- Find nearest neighbors
SELECT id FROM items
ORDER BY embedding <#> encode_to_tqvector($query::float4[], 4, 42)
LIMIT 10;
```

## Next Steps

- [Usage Guide](usage.md) — encoding parameters, index tuning, query patterns
- [Benchmarks](benchmarks.md) — performance results and methodology
- [Contributing](contributing.md) — development workflow, testing, CI
