# Usage Guide

## Encoding Parameters

`encode_to_tqvector(input, codebook_bits, rng_seed)` compresses an fp32 vector into TurboQuant format.

| Parameter | Type | Description |
| --- | --- | --- |
| `input` | `float4[]` | Raw embedding vector |
| `codebook_bits` | `integer` | Quantization depth — bits per MSE codebook entry (e.g. 4) |
| `rng_seed` | `bigint` | Seed for the random rotation matrix. Must be the same for all vectors that will be compared |

All vectors in the same index must use the same `codebook_bits` and `rng_seed`.

## The `<#>` Operator

The `<#>` operator computes the negative inner product between two `tqvector` values. The negation makes `ORDER BY ASC` return highest-similarity results first, following the pgvector convention.

```sql
SELECT * FROM items
ORDER BY embedding <#> encode_to_tqvector($query::float4[], 4, 42)
LIMIT 10;
```

## Index Tuning

The `tqhnsw` index accepts these parameters:

| Parameter | Default | Description |
| --- | --- | --- |
| `m` | 8 | Number of bidirectional links per node. Higher = better recall, more storage |
| `ef_construction` | 64 | Search width during index build. Higher = better graph quality, slower build |

At query time, `ef_search` controls the search width (trade-off between recall and latency).

```sql
-- Higher m for better recall
CREATE INDEX ON items USING tqhnsw (embedding) WITH (m=16, ef_construction=200);

-- Set ef_search for a session
SET tqhnsw.ef_search = 200;
```

## Compression Characteristics

For 1536-dimensional vectors (e.g. OpenAI `text-embedding-3-large`):

| Format | Size per vector |
| --- | --- |
| fp32 (pgvector) | 6,144 bytes |
| tqvector (4-bit) | 783 bytes |
| **Compression ratio** | **7.85x** |
