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
| TC-001 | `payload_len` returns correct byte count for 4-bit 1536-dim | FR-001-AC-3 | Assert `payload_len(1536, 4) == 772` |
| TC-002 | `payload_len` returns correct byte count for 8-bit 1536-dim | FR-001-AC-3 | Assert `payload_len(1536, 8) == 1540` |
| TC-003 | `pack`/`unpack` round-trip preserves all fields | FR-001-AC-3 | Pack `(dim, bits, seed, gamma, codes)`, unpack, compare |
| TC-004 | `format_text`/`parse_text` round-trip | FR-002-AC-1 | Format then parse, compare fields |
| TC-005 | `parse_text` rejects wrong code length | FR-002-AC-3 | Assert parse returns Err on short hex |
| TC-006 | `parse_text` rejects invalid hex | FR-002-AC-2 | Assert parse returns Err on "ZZZZ" |
| TC-007 | `unpack` rejects truncated binary | FR-003-AC-2 | Assert unpack returns Err on < 15 bytes |
| TC-008 | `log_gamma` matches known values | FR-013-AC-2 | Assert log_gamma(1.0) ≈ 0, log_gamma(5.0) ≈ ln(24) |
| TC-009 | Beta PDF integrates to 1.0 | FR-013-AC-2 | Numerical integration over [-1,1], assert sum ≈ 1.0 |
| TC-010 | Lloyd-Max 1-bit centroids are symmetric | FR-013-AC-1 | Assert sum of 2 centroids < 1e-3 |
| TC-011 | FWHT preserves vector norm | FR-013-AC-3 | Apply FWHT, compare norms, relative error < 1e-5 |
| TC-012 | SRHT rotation preserves norm | FR-013-AC-3 | Apply full SRHT, compare norms |
| TC-013 | Encode is deterministic | FR-013-AC-5 | Encode same vector twice, assert byte-identical |
| TC-014 | QJL output has correct byte count | FR-013-AC-6 | Assert QJL bytes == ceil(original_dim/8) |
| TC-015 | Encode-decode cosine similarity > 0.85 | FR-013-AC-4 | Average over 100 random 1536-dim unit vectors generated with a fixed seed |
| TC-016 | SIMD fwht matches scalar fwht | FR-014-AC-2 | Compare outputs on 1000 random inputs |
| TC-017 | SIMD score_ip matches scalar score_ip | FR-014-AC-2 | Compare outputs on 1000 random encoded pairs |
| TC-018 | Beta PDF returns 0.0 outside [-1,1] | FR-013 | Assert beta_pdf(1.5, 16) == 0.0 |
| TC-019 | `bits` validation rejects 0 and 9 | FR-004-AC-3 | Assert encode rejects out-of-range bits |
| TC-020 | Payload length ignores transform padding | FR-004-AC-4 | Assert persisted payload_len uses original_dim, not next_power_of_two(dim) |
| TC-021 | ProdQuantizer encode produces correct payload length | FR-015-AC-1 | Assert 772-byte payload for 1536-dim 4-bit |
| TC-022 | ProdQuantizer encode+decode round-trip fidelity | FR-015-AC-2 | Cosine similarity > 0.85 over 100 random vectors |
| TC-023 | Prepared-query scoring matches declared formula | FR-015-AC-3 | score_ip_encoded vs explicit FR-013 formula, within 1e-6 |
| TC-024 | Code-to-code scoring is symmetric | FR-015-AC-4 | score_ip_encoded_lite(a,b) == score_ip_encoded_lite(b,a) |
| TC-025 | MSE pack/unpack round-trip at all bit widths | FR-015-AC-5 | Test bits 1–7, random indices, assert lossless |
| TC-026 | QJL pack/unpack round-trip | FR-015-AC-5 | Random sign vectors, pack then unpack, compare |
| TC-027 | ProdQuantizer construction is deterministic | FR-015-AC-6 | Two instances with same params → identical codebooks |
| TC-028 | score_ip_encoded allocates zero heap memory | FR-015-AC-7, FR-017-AC-3 | Benchmark with allocation counter, assert 0 |
| TC-029 | LUT memory footprint ≤ 48KB for 1536-dim 4-bit | FR-017-AC-4 | Assert LUT size == dim × num_centroids × 4 bytes |
| TC-030 | SIMD qjl_bit_expand matches scalar | FR-014-AC-2 | Compare outputs on random bit vectors |
| TC-031 | Prepared-query LUT matches decoded rotated query | FR-015-AC-3 | Prepare query, compare LUT entries against decoded rotated-domain reference |
| TC-032 | ProdQuantizer cache reuses state | FR-015-AC-8 | Repeated construction requests return shared backend-local state |
| TC-033 | Code-to-code scorer ignores gamma and QJL | FR-015-AC-9 | Mutate only gamma and qjl bits, assert score_ip_encoded_lite unchanged |
| TC-034 | TqElementTuple write/read round-trip preserves all fields | FR-007-AC-2 | Construct a TqElementTuple with known values, write to a page buffer, read back, assert all fields match |
| TC-035 | Fuzz tqvector_in with random byte sequences | NFR-004 | Feed 10,000 random byte slices (lengths 0–2048) to the text input parser; assert no panic, no crash — only Ok or Err |
| TC-036 | All unsafe blocks have SAFETY comments | NFR-004 | grep for `unsafe` blocks in src/; assert every one is preceded by a `// SAFETY:` comment within 3 lines |

## Integration Tests (`cargo pgrx test`)

| TC | Description | Traces | Method |
|---|---|---|---|
| TC-101 | `tqvector` type visible in pg_type after CREATE EXTENSION | FR-001-AC-1, FR-012-AC-1 | SQL: `SELECT typname FROM pg_type WHERE typname = 'tqvector'` |
| TC-102 | Text I/O round-trip through SQL | FR-002-AC-1 | `SELECT '[dim=4,bits=4,seed=42,gamma=0.0]:...'::tqvector::text` |
| TC-103 | Text input rejects invalid hex | FR-002-AC-2 | `SELECT '[dim=4,bits=4,gamma=0.0]:ZZZZ'::tqvector` → ERROR |
| TC-104 | Binary send/recv round-trip | FR-003-AC-1 | COPY BINARY out then COPY BINARY in, compare |
| TC-105 | `encode_to_tqvector` produces storable value | FR-004-AC-1 | INSERT encoded value into tqvector column, SELECT back |
| TC-106 | `encode_to_tqvector` is deterministic | FR-004-AC-2 | Two calls, compare output bytes |
| TC-107 | `encode_to_tqvector` rejects bits=0 | FR-004-AC-3 | Assert ERROR raised |
| TC-108 | `<#>` query operator parses in ORDER BY | FR-006-AC-1 | `SELECT ... ORDER BY col <#> $q LIMIT 10` where `$q` is `float4[]` |
| TC-109 | code-to-code `<#>` overload is commutative | FR-006-AC-3 | Assert `a <#> b = b <#> a` for `(tqvector, tqvector)` |
| TC-110 | Dimension mismatch raises ERROR | FR-005-AC-2, FR-017-AC-2 | Compare tqvectors with different dim and compare a tqvector against a mismatched raw query dimension |
| TC-111 | Code-to-code inner product is symmetric | FR-005-AC-3 | Assert `ip(a,b) = ip(b,a)` |
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
| TC-122 | amoptions rejects m=0 | FR-008-AC-6 | `CREATE INDEX ... WITH (m=0)` → ERROR |
| TC-123 | amoptions accepts valid params | FR-008-AC-6 | `CREATE INDEX ... WITH (m=8, ef_construction=64)` succeeds |
| TC-124 | Bulk build: all neighbor TIDs valid | FR-008-AC-4 | After build, walk all neighbor tuples, assert each TID → valid element |
| TC-125 | Bulk build uses configured raw source column | FR-008-AC-5 | Build with `build_source_column`, verify raw-f32 build path is used and recall exceeds compressed-build fallback |
| TC-126 | Page extension: insert beyond single page | FR-007-AC-4 | Insert 100 rows (exceed single page), no errors |
| TC-127 | Concurrent inserts no deadlock | FR-007-AC-5, FR-016-AC-3 | 10 concurrent inserters for 30 seconds, no deadlock |
| TC-128 | Insert into existing index: new row reachable | FR-016-AC-1 | Insert row, immediately search, assert found |
| TC-129 | Known-vector estimator accuracy | FR-005-AC-1, FR-017-AC-1 | Benchmark code-to-code and query-to-code estimators separately against true fp32 IP |
| TC-134 | Negative wrapper functions negate base scores | FR-018-AC-1, FR-018-AC-2 | Assert each negative wrapper equals `-1` times its corresponding positive function |
| TC-130 | Multi-PG-version support | FR-012-AC-3 | `cargo pgrx test pg14`, pg15, pg16, pg17 all pass |
| TC-131 | Partition-local scan touches only one partition index | FR-009, StR-003 | Run query against one partition, assert only that partition index is scanned |
| TC-132 | Partition-local vacuum does not touch sibling partitions | FR-010, StR-003 | Vacuum one partition index, assert other partition indexes unchanged |
| TC-133 | Insert-drift statistics are queryable | FR-016-AC-4 | Read exposed metadata or stats view, assert total_live_nodes and inserted_since_rebuild are present and consistent after inserts |

## Property Tests (`proptest`)

| PT | Description | Traces | Method |
|---|---|---|---|
| PT-001 | SRHT preserves L2 norm | FR-013-AC-3 | `‖srht(x)‖ / ‖x‖ ≈ 1.0`, relative error < 1e-4, dim 2-512 |
| PT-002 | SRHT roundtrip | FR-013-AC-3 | `inverse_srht(srht(x)) ≈ x`, element-wise error < 1e-4, dim 2-512 |
| PT-003 | SRHT roundtrip at real-world dims | FR-013-AC-3 | Same as PT-002 at dims 384, 768, 1024, 1536 |
| PT-004 | MSE pack/unpack roundtrip | FR-015-AC-5 | Arbitrary dim (1-2048), bits (1-7), random indices |
| PT-005 | QJL pack/unpack roundtrip | FR-015-AC-5 | Arbitrary dim (1-4096), random bools |
| PT-006 | Encode determinism | FR-013-AC-5, FR-015-AC-6 | Same input → byte-identical payload, dims 32-256 |
| PT-007 | score_ip_codes_lite symmetry | FR-015-AC-4 | `score(a,b) == score(b,a)`, dims 32-256 |
| PT-008 | payload_len matches actual | FR-015-AC-1 | `pack_payload(encode(v)).len() == payload_len(dim, bits)` |
| PT-009 | score_ip_encoded == score_ip_from_parts | FR-015-AC-3 | Same data, both paths, within 1e-6 |
| PT-010 | decode_approximate bounded cosine error | FR-013-AC-4 | Cosine similarity > threshold (0.3 for 2-bit, 0.6 for 4+bit) |
| PT-011 | TqElementTuple encode/decode roundtrip | FR-007-AC-2 | Random fields, code_len 64-1536 |
| PT-012 | TqNeighborTuple encode/decode roundtrip | FR-007-AC-2 | Random count 1-63 |
| PT-013 | MetadataPage encode/decode roundtrip | FR-007-AC-1 | Random metadata fields |
| PT-014 | ItemPointer encode/decode roundtrip | FR-007-AC-2 | Full u32/u16 range |
| PT-015 | element_tuple_encoded_len correctness | FR-007-AC-2 | `encode().len() == encoded_len(code_len)` |

## Miri Tests (UB Detection)

| MI | Description | Traces | Method |
|---|---|---|---|
| MI-001 | Encode/decode roundtrip | FR-015 | dim=8 4-bit encode → decode_approximate, no UB |
| MI-002 | MSE pack/unpack | FR-015-AC-5 | dim=16 3-bit, random indices, no UB |
| MI-003 | QJL pack/unpack | FR-015-AC-5 | dim=16, random signs, no UB |
| MI-004 | score_ip_encoded end-to-end | FR-015-AC-3 | dim=8 4-bit, no UB |
| MI-005 | score_ip_codes_lite | FR-015-AC-4 | dim=8 4-bit, no UB |
| MI-006 | fwht_in_place | FR-013-AC-3 | len=4, no UB |
| MI-007 | orthonormal_fwht_in_place | FR-013-AC-3 | len=8, no UB |
| MI-008 | ItemPointer roundtrip | FR-007-AC-2 | encode_into + decode, no UB |
| MI-009 | TqElementTuple roundtrip | FR-007-AC-2 | Small tuple, no UB |
| MI-010 | TqNeighborTuple roundtrip | FR-007-AC-2 | count=3, no UB |
| MI-011 | MetadataPage roundtrip | FR-007-AC-1 | encode + decode, no UB |

## Fuzz Targets (`cargo-fuzz`)

| FZ | Description | Traces | Method |
|---|---|---|---|
| FZ-001 | parse_text | NFR-004, FR-002 | Arbitrary UTF-8 strings through text parser, no panic |
| FZ-002 | unpack_mse | NFR-004, FR-015 | Structure-aware: first byte = dim, second = bits, rest = packed data |
| FZ-003 | element_tuple_decode | NFR-004, FR-007 | Arbitrary bytes, code_len derived from first byte × 4 |
| FZ-004 | neighbor_tuple_decode | NFR-004, FR-007 | Arbitrary bytes through TqNeighborTuple::decode |

## Layout Assertions (`size_of_assertions`)

| LA | Description | Traces | Expected |
|---|---|---|---|
| LA-001 | payload_len(1536, 4) | FR-001-AC-3 | 772 |
| LA-002 | payload_len(1536, 2) | FR-001-AC-3 | 388 |
| LA-003 | payload_len(1536, 3) | FR-001-AC-3 | Locked value |
| LA-004 | payload_len(1536, 6) | FR-001-AC-3 | Locked value |
| LA-005 | payload_len(1536, 8) | FR-001-AC-3 | 1540 |
| LA-006 | mse_code_len(1536, 4) | FR-015 | 576 |
| LA-007 | qjl_code_len(1536) | FR-015 | 192 |
| LA-008 | ITEM_POINTER_BYTES | FR-007 | 6 |
| LA-009 | mem::size_of::\<ItemPointer\>() | FR-007 | 8 |
| LA-010 | PAGE_HEADER_BYTES | FR-007 | 24 |
| LA-011 | HEAPTID_INLINE_CAPACITY | FR-007 | 10 |
| LA-012 | element_tuple_encoded_len(768) | FR-007 | Locked value |
| LA-013 | Compression ratio ≥ 7.8x | NFR-002 | 1536-dim 4-bit vs raw fp32 |

## Benchmarks

| BC | Description | Traces | Target | Status |
|---|---|---|---|---|
| BC-001 | HNSW top-10 latency (50K × 1536, 4-bit, m=8) | NFR-001 | p50 < 5ms, p99 < 15ms | Blocked (scan) |
| BC-002 | Sequential scan throughput (compressed-domain scoring) | NFR-001 | Report scores/sec and rows/sec on representative hardware | Blocked (scan) |
| BC-003 | Single `tqvector_inner_product` and `tqvector_query_inner_product` latency | NFR-001, FR-005, FR-017 | Report both latencies on representative hardware | Blocked (scan) |
| BC-004 | Index size (50K × 1536, 4-bit, m=8) | NFR-002 | Report payload bytes, tuple bytes, and total relation size | Blocked (scan) |
| BC-005 | Recall@10 (50K × 1536, m=8, ef=128) | NFR-003 | ≥ 89% | Blocked (scan) |
| BC-006 | Recall@10 (50K × 1536, m=8, ef=200) | NFR-003 | ≥ 93% | Blocked (scan) |
| BC-007 | Recall@10 (50K × 1536, m=16, ef=200) | NFR-003 | ≥ 97% | Blocked (scan) |
| BC-008 | FWHT AVX2 vs scalar throughput (dim=2048) | FR-014-AC-4 | ≥ 3x speedup | Blocked (SIMD) |
| BC-009 | 1M vectors disk usage (1536-dim, 4-bit) | NFR-002 | Report code bytes and total on-disk index bytes separately | Blocked (scan) |
| BC-010 | score_ip_encoded throughput (1536-dim 4-bit) | NFR-001, FR-015-AC-7 | > 200K scores/sec | **Measured: ~95K/s** |
| BC-011 | Recall drift after incremental inserts | NFR-003, FR-016 | Report recall vs fraction of rows inserted since bulk build | Blocked (insert) |
| BC-012 | Truncated-tail vs full-tail quality comparison | NFR-003, ADR-007 | Report Recall@10/100, NDCG@10, rank correlation, and storage delta | Blocked (scan) |
| BC-013 | Raw-query vs code-to-code scorer comparison | NFR-003, ADR-007 | Report quality gap and latency gap on identical query sets | Blocked (scan) |
| BC-014 | MSE+QJL vs MSE-only ablation | NFR-003, ADR-007 | Report quality gain attributable to the QJL term | Blocked (scan) |
| BC-015 | Warm-cache vs cold-cache HNSW latency | NFR-001 | Report both latency profiles under the same dataset and settings | Blocked (scan) |
| BC-016 | Post-vacuum recall (50K × 1536, 4-bit, m=8, 10% deleted) | NFR-003, FR-010 | After deleting 10% of rows and running VACUUM, recall@10 SHALL be ≥ 80% of pre-vacuum recall using NFR-003 methodology | Blocked (vacuum) |
| BC-017 | Quantizer-level recall — uniform corpus (50K × 1536, 4-bit) | NFR-003 | Report Recall@1/10/100, NDCG@10, MAE, Spearman rho | **Harness ready** |
| BC-018 | Quantizer-level recall — clustered corpus (10K × 1536, 50 clusters) | NFR-003 | Report same metrics as BC-017 on realistic clustered data | **Harness ready** |
| BC-019 | Near-duplicate ranking preservation | NFR-003 | Quantized ranking preserves true nearest at angles 0.01-0.2 rad | **Harness ready** |
| BC-020 | Bit-width sensitivity — uniform and clustered | NFR-003 | Report recall across bits 2-8 on both distributions | **Harness ready** |
| BC-021 | Dimension sensitivity (128-1536, 4-bit) | NFR-003 | Report recall across dims on uniform corpus | **Harness ready** |
| BC-022 | DataPage insert/read element throughput | FR-007, NFR-001 | Report ns/op for insert and read at code_len 192, 768 | **Measured** |
| BC-023 | DataPage insert/read neighbor throughput | FR-007, NFR-001 | Report ns/op for insert and read at count 16, 32 | **Measured** |
| BC-024 | score_ip_from_parts throughput | NFR-001, FR-015 | Report µs/score across dim/bit configs | **Measured** |
| BC-025 | score_ip_encoded_lite throughput | NFR-001, FR-015 | Report µs/score across dim/bit configs | **Measured** |
| BC-026 | decode_approximate throughput | NFR-001, FR-015 | Report µs/decode at 1536/4-bit and 3072/4-bit | **Measured** |
| BC-027 | Instruction count regression — scoring hot loop | NFR-001 | iai-callgrind: score_ip_encoded, codes_lite, from_parts at 1536/4-bit | **Harness ready** |
| BC-028 | Instruction count regression — hadamard | NFR-001 | iai-callgrind: fwht_in_place at 2048, 4096 | **Harness ready** |
| BC-029 | Instruction count regression — bitpack | NFR-001 | iai-callgrind: pack/unpack MSE, pack QJL at 1536/3-bit | **Harness ready** |
| BC-030 | Zero-allocation scoring verification | FR-015-AC-7 | dhat: 10Kx100 score_ip_encoded, zero heap allocations in profiled region | **Harness ready** |
| BC-031 | Encode allocation profile | FR-015 | dhat: 1000x encode at 1536/4-bit, report allocation count and bytes | **Harness ready** |

## Benchmark Execution Rules

- All benchmark reports SHALL identify the dataset, row count, dimensionality, query count, random seed, hardware, PostgreSQL version, compiler profile, and relevant PostgreSQL settings.
- All quality benchmarks SHALL use brute-force exact fp32 inner product over the same raw vectors as ground truth.
- All variant comparisons SHALL use the same query set, the same HNSW hyperparameters, and the same hardware/configuration.
- Drift benchmarks SHALL report results at a minimum after 0%, 5%, 10%, and 20% of rows have been inserted since the last bulk build or REINDEX.
- Tail-truncation benchmarks SHALL compare the current persisted layout against a tail-retaining offline reference variant.
- Ablation benchmarks SHALL compare the full declared scorer against an MSE-only variant with `gamma = 0` and QJL ignored.
- Latency benchmarks SHALL report warm-cache and cold-cache results separately when feasible.
- Any benchmark used to justify a product decision SHALL be reproducible from a checked-in SQL script, harness, or benchmark command line.

## Coverage Summary

| Requirement | Test Cases |
|---|---|
| FR-001 | TC-001, TC-002, TC-003, TC-101, LA-001 to LA-005 |
| FR-002 | TC-004, TC-005, TC-006, TC-102, TC-103, FZ-001 |
| FR-003 | TC-007, TC-104 |
| FR-004 | TC-019, TC-020, TC-105, TC-106, TC-107 |
| FR-005 | TC-110, TC-111, TC-129, BC-003 |
| FR-006 | TC-108, TC-109, TC-114 |
| FR-007 | TC-034, TC-117, TC-126, TC-127, PT-011 to PT-015, MI-008 to MI-011, FZ-003, FZ-004, LA-008 to LA-012, BC-022, BC-023 |
| FR-008 | TC-112, TC-122, TC-123, TC-124, TC-125 |
| FR-009 | TC-113, TC-114, TC-120, TC-121, TC-131 |
| FR-010 | TC-115, TC-118, TC-132, BC-016 |
| FR-011 | TC-119 |
| FR-012 | TC-101, TC-116, TC-130 |
| FR-013 | TC-008 to TC-015, TC-018, PT-001 to PT-003, PT-006, PT-010, MI-001, MI-006, MI-007 |
| FR-014 | TC-016, TC-017, TC-030, BC-008 |
| FR-015 | TC-021 to TC-028, TC-031 to TC-033, PT-004 to PT-009, MI-001 to MI-005, FZ-002, LA-006, LA-007, BC-010, BC-024 to BC-031 |
| FR-016 | TC-127, TC-128, TC-133, BC-011 |
| FR-017 | TC-028, TC-029, TC-110, TC-129, BC-003 |
| FR-018 | TC-134 |
| NFR-001 | BC-001, BC-002, BC-003, BC-010, BC-015, BC-022 to BC-029 |
| NFR-002 | BC-004, BC-009, LA-013 |
| NFR-003 | BC-005 to BC-007, BC-011 to BC-014, BC-017 to BC-021 |
| NFR-004 | TC-035, TC-036, TC-118, TC-119, FZ-001 to FZ-004, MI-001 to MI-011, all unit tests (no panic) |
| NFR-005 | CI pipeline (fmt, clippy, test, pgrx test, deny, proptest, layout-check, miri, bench-action) |
