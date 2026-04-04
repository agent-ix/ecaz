---
artifact_type: test-matrix
name: tqvector
---
# Test Matrix — tqvector

Bidirectional traceability between requirements and test cases.

---

## Unit Tests (`cargo test`)

| TC | Description | Traces | Method |
|---|---|---|---|
| TC-001 | `code_len` returns correct byte count for 4-bit 1536-dim | FR-001-AC-3 | Assert `code_len(1536, 4) == 768` |
| TC-002 | `code_len` returns correct byte count for 8-bit 1536-dim | FR-001-AC-3 | Assert `code_len(1536, 8) == 1536` |
| TC-003 | `pack`/`unpack` round-trip preserves all fields | FR-001-AC-3 | Pack (4, 4, 42, codes), unpack, compare |
| TC-004 | `format_text`/`parse_text` round-trip | FR-002-AC-1 | Format then parse, compare fields |
| TC-005 | `parse_text` rejects wrong code length | FR-002-AC-3 | Assert parse returns Err on short hex |
| TC-006 | `parse_text` rejects invalid hex | FR-002-AC-2 | Assert parse returns Err on "ZZZZ" |
| TC-007 | `unpack` rejects truncated binary | FR-003-AC-2 | Assert unpack returns Err on < 11 bytes |
| TC-008 | `log_gamma` matches known values | FR-013-AC-2 | Assert log_gamma(1.0) ≈ 0, log_gamma(5.0) ≈ ln(24) |
| TC-009 | Beta PDF integrates to 1.0 | FR-013-AC-2 | Numerical integration over [-1,1], assert sum ≈ 1.0 |
| TC-010 | Lloyd-Max 1-bit centroids are symmetric | FR-013-AC-1 | Assert sum of 2 centroids < 1e-3 |
| TC-011 | FWHT preserves vector norm | FR-013-AC-3 | Apply FWHT, compare norms, relative error < 1e-5 |
| TC-012 | SRHT rotation preserves norm | FR-013-AC-3 | Apply full SRHT, compare norms |
| TC-013 | Encode is deterministic | FR-013-AC-5 | Encode same vector twice, assert byte-identical |
| TC-014 | QJL output has correct byte count | FR-013-AC-6 | Assert QJL bytes == ceil(dim/8) |
| TC-015 | Encode-decode cosine similarity > 0.85 | FR-013-AC-4 | Average over 100 random 1536-dim unit vectors |
| TC-016 | SIMD fwht matches scalar fwht | FR-014-AC-2 | Compare outputs on 1000 random inputs |
| TC-017 | SIMD score_ip matches scalar score_ip | FR-014-AC-2 | Compare outputs on 1000 random encoded pairs |
| TC-018 | Beta PDF returns 0.0 outside [-1,1] | FR-013 | Assert beta_pdf(1.5, 16) == 0.0 |
| TC-019 | `bits` validation rejects 0 and 9 | FR-004-AC-3 | Assert encode rejects out-of-range bits |
| TC-020 | Code length matches dim after padding | FR-004-AC-4 | Assert code_len uses next_power_of_two(dim) |
| TC-021 | ProdQuantizer encode produces correct code length | FR-015-AC-1 | Assert 768 bytes for 1536-dim 4-bit |
| TC-022 | ProdQuantizer encode+decode round-trip fidelity | FR-015-AC-2 | Cosine similarity > 0.85 over 100 random vectors |
| TC-023 | LUT scoring matches brute-force decoded IP | FR-015-AC-3 | score_ip_encoded vs decoded dot product, within 1e-6 |
| TC-024 | Code-to-code scoring is symmetric | FR-015-AC-4 | score_ip_encoded_lite(a,b) == score_ip_encoded_lite(b,a) |
| TC-025 | MSE pack/unpack round-trip at all bit widths | FR-015-AC-5 | Test bits 1–7, random indices, assert lossless |
| TC-026 | QJL pack/unpack round-trip | FR-015-AC-5 | Random sign vectors, pack then unpack, compare |
| TC-027 | ProdQuantizer construction is deterministic | FR-015-AC-6 | Two instances with same params → identical codebooks |
| TC-028 | score_ip_encoded allocates zero heap memory | FR-015-AC-7 | Benchmark with allocation counter, assert 0 |
| TC-029 | LUT memory footprint ≤ 48KB for 1536-dim 4-bit | FR-005-AC-5 | Assert LUT size == dim × num_centroids × 4 bytes |
| TC-030 | SIMD qjl_bit_expand matches scalar | FR-014-AC-2 | Compare outputs on random bit vectors |

## Integration Tests (`cargo pgrx test`)

| TC | Description | Traces | Method |
|---|---|---|---|
| TC-101 | `tqvector` type visible in pg_type after CREATE EXTENSION | FR-001-AC-1, FR-012-AC-1 | SQL: `SELECT typname FROM pg_type WHERE typname = 'tqvector'` |
| TC-102 | Text I/O round-trip through SQL | FR-002-AC-1 | `SELECT '[dim=4,bits=4,seed=42]:...'::tqvector::text` |
| TC-103 | Text input rejects invalid hex | FR-002-AC-2 | `SELECT '[dim=4,bits=4]:ZZZZ'::tqvector` → ERROR |
| TC-104 | Binary send/recv round-trip | FR-003-AC-1 | COPY BINARY out then COPY BINARY in, compare |
| TC-105 | `encode_to_tqvector` produces storable value | FR-004-AC-1 | INSERT encoded value into tqvector column, SELECT back |
| TC-106 | `encode_to_tqvector` is deterministic | FR-004-AC-2 | Two calls, compare output bytes |
| TC-107 | `encode_to_tqvector` rejects bits=0 | FR-004-AC-3 | Assert ERROR raised |
| TC-108 | `<#>` operator parses in ORDER BY | FR-006-AC-1 | `SELECT ... ORDER BY col <#> $q LIMIT 10` |
| TC-109 | `<#>` operator is commutative | FR-006-AC-3 | Assert `a <#> b = b <#> a` |
| TC-110 | Dimension mismatch raises ERROR | FR-005-AC-2 | Compare tqvectors with different dim |
| TC-111 | Inner product is symmetric | FR-005-AC-3 | Assert `ip(a,b) = ip(b,a)` |
| TC-112 | CREATE INDEX USING tqhnsw succeeds | FR-008-AC-1 | Build index on 1000-row table |
| TC-113 | Index scan returns top-k results | FR-009-AC-1 | `ORDER BY <#> LIMIT 10`, assert 10 rows |
| TC-114 | EXPLAIN shows tqhnsw index scan | FR-009-AC-3, FR-006-AC-2 | Check EXPLAIN output |
| TC-115 | DELETE + VACUUM removes from results | FR-010-AC-1 | Delete row, VACUUM, search, assert absent |
| TC-116 | DROP EXTENSION CASCADE cleans up | FR-012-AC-2 | Check pg_type, pg_operator, pg_am after drop |
| TC-117 | Metadata page readable after CREATE INDEX | FR-007-AC-1 | Read page 0, validate M and ef_construction |
| TC-118 | Concurrent INSERT + VACUUM no errors | FR-010-AC-3 | Run VACUUM and INSERT concurrently for 10s |
| TC-119 | Crash recovery: REINDEX after kill -9 | FR-011-AC-1 | Build index, kill backend, restart, REINDEX |
| TC-120 | GUC ef_search affects results | FR-009-AC-4 | Compare recall at ef_search=10 vs ef_search=200 |
| TC-121 | GUC ef_search is session-settable | FR-009-AC-5 | `SET tqhnsw.ef_search = 200` succeeds |
| TC-122 | amoptions rejects m=0 | FR-008-AC-7 | `CREATE INDEX ... WITH (m=0)` → ERROR |
| TC-123 | amoptions accepts valid params | FR-008-AC-7 | `CREATE INDEX ... WITH (m=8, ef_construction=64)` succeeds |
| TC-124 | Bulk build: all neighbor TIDs valid | FR-008-AC-5 | After build, walk all neighbor tuples, assert each TID → valid element |
| TC-125 | Bulk build uses f32 distance (quality check) | FR-008-AC-6 | Build index, verify recall > 85% (wouldn't reach this if built from lossy codes) |
| TC-126 | Page extension: insert beyond single page | FR-007-AC-4 | Insert 100 rows (exceed single page), no errors |
| TC-127 | Concurrent inserts no deadlock | FR-007-AC-5 | 10 concurrent inserters for 30 seconds, no deadlock |
| TC-128 | Insert into existing index: new row reachable | FR-008-AC-2 | Insert row, immediately search, assert found |
| TC-129 | Known-vector inner product accuracy | FR-005-AC-1 | Encode known vectors, assert IP within 15% of true |
| TC-130 | Multi-PG-version support | FR-012-AC-3 | `cargo pgrx test pg14`, pg15, pg16, pg17 all pass |

## Benchmarks

| BC | Description | Traces | Target |
|---|---|---|---|
| BC-001 | HNSW top-10 latency (50K × 1536, 4-bit, m=8) | NFR-001 | p50 < 5ms, p99 < 15ms |
| BC-002 | Sequential scan top-10 latency (500K × 1536, 4-bit) | NFR-001 | < 3ms |
| BC-003 | Single `tqvector_inner_product` call latency | NFR-001 | < 5μs |
| BC-004 | Index size (50K × 1536, 4-bit, m=8) | NFR-002 | ≤ 34 MB |
| BC-005 | Recall@10 (50K × 1536, m=8, ef=128) | NFR-003 | ≥ 89% |
| BC-006 | Recall@10 (50K × 1536, m=8, ef=200) | NFR-003 | ≥ 93% |
| BC-007 | Recall@10 (50K × 1536, m=16, ef=200) | NFR-003 | ≥ 97% |
| BC-008 | FWHT AVX2 vs scalar throughput (dim=2048) | FR-014-AC-4 | ≥ 3x speedup |
| BC-009 | 1M vectors disk usage (1536-dim, 4-bit) | NFR-002 | < 1 GB |
| BC-010 | score_ip_encoded throughput (1536-dim 4-bit) | NFR-001, FR-015-AC-7 | > 200K scores/sec |

## Coverage Summary

| Requirement | Test Cases |
|---|---|
| FR-001 | TC-001, TC-002, TC-003, TC-101 |
| FR-002 | TC-004, TC-005, TC-006, TC-102, TC-103 |
| FR-003 | TC-007, TC-104 |
| FR-004 | TC-019, TC-020, TC-105, TC-106, TC-107 |
| FR-005 | TC-029, TC-110, TC-111, TC-129, BC-003 |
| FR-006 | TC-108, TC-109, TC-114 |
| FR-007 | TC-117, TC-126, TC-127 |
| FR-008 | TC-112, TC-122, TC-123, TC-124, TC-125, TC-128 |
| FR-009 | TC-113, TC-114, TC-120, TC-121 |
| FR-010 | TC-115, TC-118 |
| FR-011 | TC-119 |
| FR-012 | TC-101, TC-116, TC-130 |
| FR-013 | TC-008, TC-009, TC-010, TC-011, TC-012, TC-013, TC-014, TC-015, TC-018 |
| FR-014 | TC-016, TC-017, TC-030, BC-008 |
| FR-015 | TC-021, TC-022, TC-023, TC-024, TC-025, TC-026, TC-027, TC-028, BC-010 |
| NFR-001 | BC-001, BC-002, BC-003 |
| NFR-002 | BC-004, BC-009 |
| NFR-003 | BC-005, BC-006, BC-007 |
| NFR-004 | TC-119, TC-118, all unit tests (no panic) |
| NFR-005 | CI pipeline (fmt, clippy, test, pgrx test, deny) |
