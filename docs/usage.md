# Usage Guide

## Row Types

Use `ecvector(dim)` for normal tables. It is the canonical exact/raw row type
used by `ec_hnsw`, `ec_ivf`, and `ec_diskann`.

```sql
CREATE TABLE items (
    id bigint generated always as identity primary key,
    embedding ecvector(1536)
);
```

`tqvector` remains available as an explicit TurboQuant artifact/debugging type.
New applications should prefer `ecvector` unless they are testing a specific
compressed artifact surface.

## Encoding

`encode_to_ecvector(input, codebook_bits, rng_seed)` stores an fp32 vector in
the canonical row format. The current canonical quantizer defaults are
`codebook_bits = 4` and `rng_seed = 42`; `encode_to_ecvector` rejects other
values.

```sql
INSERT INTO items (embedding)
VALUES (encode_to_ecvector($1::float4[], 4, 42));
```

`encode_to_tqvector(input, codebook_bits, rng_seed)` is the corresponding
TurboQuant artifact encoder.

## Querying

The `<#>` operator computes negative inner product. The negation follows the
pgvector convention: `ORDER BY ASC` returns highest-similarity rows first.

```sql
SELECT id
FROM items
ORDER BY embedding <#> $1::float4[]
LIMIT 10;
```

## HNSW

`ec_hnsw` is the default general-purpose graph index.

```sql
CREATE INDEX items_hnsw_idx
ON items USING ec_hnsw (embedding ecvector_ip_ops)
WITH (
    m = 8,
    ef_construction = 64,
    storage_format = 'turboquant'
);
```

| Knob | Default | Use |
| --- | ---: | --- |
| `m` | 8 | Graph degree per layer. Higher usually improves recall and storage cost. |
| `ef_construction` | 64 | Build-time search width. Higher usually improves graph quality and build cost. |
| `ef_search` | 40 | Relation default for scan width. |
| `storage_format` | `turboquant` | `turboquant` or `pq_fastscan`. |

Override scan width for a session:

```sql
SET ec_hnsw.ef_search = 200;
```

## IVF

`ec_ivf` is an opt-in posting-list index. It trains centroids, assigns each row
to one list, scans the selected lists, and can rerank candidates from heap f32
values.

```sql
CREATE INDEX items_ivf_idx
ON items USING ec_ivf (embedding ecvector_ip_ops)
WITH (
    nlists = 128,
    nprobe = 48,
    storage_format = 'pq_fastscan',
    pq_group_size = 8,
    rerank = 'heap_f32',
    rerank_width = 500
);
```

| Knob | Default | Use |
| --- | ---: | --- |
| `nlists` | 0 | Number of centroid posting lists. `0` auto-selects from row count. |
| `nprobe` | 0 | Lists to scan. `0` uses the relation default resolution path. |
| `storage_format` | `auto` | `auto`, `turboquant`, `pq_fastscan`, or `rabitq`. |
| `pq_group_size` | 0 | PQ-FastScan group size. Use `8` for the measured high-dimensional local profile. |
| `rerank` | `auto` | `auto`, `off`, or `heap_f32`. |
| `rerank_width` | 0 | Candidate frontier width for heap rerank. |
| `training_sample_rows` | 0 | Training sample limit. `0` uses the automatic sampler. |
| `posting_slack_percent` | 0 | Extra posting-list page slack for churn-heavy workloads. |

Override scan knobs for a session:

```sql
SET ec_ivf.nprobe = 48;
SET ec_ivf.rerank_width = 500;
```

Current local evidence keeps `storage_format = 'auto'` unchanged. For larger
high-dimensional IVF surfaces where speed and index size dominate, the measured
recommendation is explicit `storage_format = 'pq_fastscan', pq_group_size = 8`.

## DiskANN

`ec_diskann` is an opt-in DiskANN/Vamana-style graph index. It is separate from
HNSW and IVF so disk-resident graph behavior can be measured directly. DiskANN
v0 requires unit-normalized source vectors because its exact graph distance
wrapper uses `1 - inner_product`.

```sql
CREATE INDEX items_diskann_idx
ON items USING ec_diskann (embedding ecvector_diskann_ip_ops)
WITH (
    graph_degree = 32,
    build_list_size = 100,
    list_size = 100,
    rerank_budget = 64,
    alpha = 1.2
);
```

| Knob | Default | Use |
| --- | ---: | --- |
| `graph_degree` | 32 | Vamana neighbor count. |
| `build_list_size` | 100 | Build-time search breadth. |
| `list_size` | 100 | Relation default scan breadth. |
| `rerank_budget` | 64 | Exact heap-rerank candidate budget. |
| `top_k` | 10 | Persisted top-k planning default. |
| `alpha` | 1.2 | Vamana prune alpha. |
| `storage_format` | `pq_fastscan` | Current DiskANN storage format. |

Override scan breadth and prefilter behavior for a session:

```sql
SET ec_diskann.list_size = 200;
SET ec_diskann.prefilter_kind = 'binary_sidecar';
```

`ec_diskann.prefilter_kind = 'auto'` uses the persisted binary sidecar when it
is present and falls back to grouped-PQ. `grouped_pq` is retained as an
emergency rollback path.

## Choosing An Index

| Access method | Best fit | Notes |
| --- | --- | --- |
| `ec_hnsw` | General-purpose ANN graph search | Default path and broadest operational baseline. |
| `ec_ivf` | Posting-list experiments, high-ingest tradeoffs, quantizer comparisons | Local v1 lane is landed; product claims need dedicated hardware. |
| `ec_diskann` | Disk-resident graph research and DiskANN/Vamana comparisons | Local Task 29 baseline is landed; low-L latency work remains future structural work. |
| `ec_spire` | Partitioned local and distributed IVF-family search | RaBitQ is the first remote-serving storage profile; product-scale claims need controlled evidence. |

## Operator CLI

Use the `ecaz` CLI for repeatable corpus setup, benchmarks, comparisons, stress
harnesses, and local development helpers:

```bash
cargo install --path crates/ecaz-cli
ecaz corpus list
ecaz corpus inspect --prefix ec_real_10k
ecaz bench recall --prefix ec_real_10k --profile ec_hnsw
ecaz bench latency --prefix ec_real_10k --profile ec_hnsw
```

The CLI is profile-aware for `ec_hnsw`, `ec_ivf`, `ec_diskann`, and
`ec_spire`, and accepts the standard PostgreSQL connection flags
(`--database`, `--host`, `--port`, `--user`, `--password`) plus libpq
environment fallbacks. For review evidence, pass
`--log-file review/<topic>/artifacts/<run>.log` so command output is stored with
the packet.

See the [Operator CLI README](../crates/ecaz-cli/README.md) for the full
command surface.

Benchmark evidence should follow the
[Benchmark Reporting Standard](benchmark-reporting-standard.md), which defines
the common fields for access-method, quantizer, storage-format, and option-set
comparisons.

## Compression Characteristics

For 1536-dimensional vectors:

| Format | Size per vector |
| --- | ---: |
| fp32 | 6,144 bytes |
| `tqvector` 4-bit artifact | 783 bytes |
| Compression ratio | 7.85x |
