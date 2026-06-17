---
id: NFR-002
title: Storage Compression
type: NFR
status: APPROVED
traces:
  - StR-001
---
# NFR-002: Storage Compression

## Statement

Ecaz SHALL store compressed vectors and indexes within the size and compression-ratio targets below, reported with full on-disk accounting.

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
|--------|--------|-----------|--------|
| Per-vector tqvector size (1536-dim, 4-bit datum) | 783 bytes | <= 783 bytes | Datum Inspection |
| Compression ratio vs raw fp32 (6,144 bytes) | >= 7.8x | >= 7.8x | Analysis |
| Full-node on-disk index size (1M × 1536, 4-bit) | Reported with full accounting | Reported | `pg_relation_size` Measurement |


Report `pg_relation_size(index_oid)` after bulk loading known row counts.

## Verification

`pg_relation_size(index_oid)` is reported after bulk loading known row counts with full element-tuple, neighbor-tuple, line-pointer, page-header, and fragmentation accounting; per-vector size and compression ratio are derived from the datum layout and asserted against threshold.

