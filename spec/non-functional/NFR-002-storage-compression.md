---
id: NFR-002
title: Storage Compression
type: non-functional-requirement
artifact_type: NFR
quality_attribute: performance_efficiency
status: APPROVED
traces:
  - StR-001
---
# NFR-002: Storage Compression

## Statement

At 1536-dim, 4-bit, per-vector storage SHALL achieve a compression ratio of
at least 7.8x versus raw fp32, and index size SHALL be benchmarked and
reported with full-node accounting as defined below.

### Index Size Reporting

At 1536-dim, 4-bit, index size SHALL be benchmarked and reported as:
- encoded payload bytes
- total element-tuple bytes
- total neighbor-tuple bytes
- total relation size on disk

Any comparison against pgvector or other baselines SHALL use the same row count, `m`, and measurement method.

### Per-Vector Storage

- Raw fp32: 6,144 bytes
- tqvector 4-bit datum: 783 bytes total (11-byte datum prefix + 772-byte quantized payload = 4-byte gamma + 768-byte code bytes)
- Compression ratio: ≥ 7.8x

### Accounting Rules

All reported index-size targets SHALL include:
- element tuples
- neighbor tuples
- line pointers and page headers
- free-space fragmentation after representative insert/delete workloads

### Disk-Level Target

1 million vectors at 1536-dim, 4-bit SHALL be reported with full-node accounting as defined above. Any published headline size claim SHALL distinguish compressed-code bytes from total on-disk index bytes.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| tqvector 4-bit per-vector datum size (1536-dim) | 783 bytes total (11-byte datum prefix + 772-byte quantized payload) | 783 bytes | per-vector storage accounting |
| Compression ratio vs raw fp32 (6,144 bytes per vector) | >= 7.8x | 7.8x | per-vector storage accounting |
| Index size at 1536-dim, 4-bit (encoded payload, element-tuple, neighbor-tuple, total relation bytes) | benchmarked and reported | reported with full-node accounting | `pg_relation_size(index_oid)` after bulk loading known row counts |
| 1M-vector on-disk index size (1536-dim, 4-bit) | reported with full-node accounting | compressed-code bytes distinguished from total on-disk index bytes | `pg_relation_size(index_oid)` after bulk loading known row counts |

Report `pg_relation_size(index_oid)` after bulk loading known row counts.

## Verification

Compliance is checked by reporting `pg_relation_size(index_oid)` after bulk
loading known row counts, applying the accounting rules above (element tuples,
neighbor tuples, line pointers and page headers, free-space fragmentation
after representative insert/delete workloads). Any comparison against pgvector
or other baselines is verified to use the same row count, `m`, and measurement
method, and published headline size claims are checked to distinguish
compressed-code bytes from total on-disk index bytes.
