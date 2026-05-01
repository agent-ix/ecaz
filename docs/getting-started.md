# Getting Started

## Prerequisites

- [Rust](https://rustup.rs/) stable toolchain
- [cargo-pgrx](https://github.com/pgcentralfoundation/pgrx) 0.17:
  `cargo install cargo-pgrx@0.17`
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

Connect to PostgreSQL and create a small table with the canonical `ecvector`
row type:

```sql
CREATE EXTENSION ecaz;

CREATE TABLE items (
    id bigint generated always as identity primary key,
    embedding ecvector(4)
);

INSERT INTO items (embedding)
VALUES
    (encode_to_ecvector(ARRAY[1.0, 0.0, 0.0, 0.0]::float4[], 4, 42)),
    (encode_to_ecvector(ARRAY[0.0, 1.0, 0.0, 0.0]::float4[], 4, 42)),
    (encode_to_ecvector(ARRAY[-1.0, 0.0, 0.0, 0.0]::float4[], 4, 42));

CREATE INDEX items_hnsw_idx
ON items USING ec_hnsw (embedding ecvector_ip_ops)
WITH (m = 8, ef_construction = 64);

SELECT id
FROM items
ORDER BY embedding <#> ARRAY[1.0, 0.0, 0.0, 0.0]::float4[]
LIMIT 2;
```

`<#>` is negative inner-product distance, so `ORDER BY ... ASC` returns the
highest inner-product matches first.

## Other Index Types

Ecaz also includes opt-in IVF and DiskANN access methods.

```sql
CREATE INDEX items_ivf_idx
ON items USING ec_ivf (embedding ecvector_ip_ops)
WITH (nlists = 2, nprobe = 1, storage_format = 'turboquant');
```

The sample rows above are unit-normalized, so they can also be used with
DiskANN. DiskANN currently validates this contract because its v0 graph
distance wrapper preserves `<#>` ordering only for unit-normalized vectors:

```sql
CREATE INDEX items_diskann_idx
ON items USING ec_diskann (embedding ecvector_diskann_ip_ops)
WITH (graph_degree = 32, build_list_size = 100, list_size = 100);
```

## Next Steps

- [Usage Guide](usage.md) - encoding, index choices, and tuning knobs
- [Benchmarks](benchmarks.md) - local results and methodology
- [Operator CLI](../crates/ecaz-cli/README.md) - corpus loading, benchmarks, comparisons, and dev helpers
- [Contributing](contributing.md) - development workflow, testing, CI
