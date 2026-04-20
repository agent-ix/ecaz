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
| TC-037 | ReadStream callback signatures and state carriers match FR-019 | FR-019 | Unit-test graph and linear callback signatures plus `GraphPrefetchState` / `LinearPrefetchState` exhaustion behavior |
| TC-043 | ReadStream callbacks return blocks then end-of-stream | FR-019 | Unit-test pure graph and linear callback helpers so PG18 bindings only need to translate `EndOfStream` into `InvalidBlockNumber` |
| TC-047 | ReadStream state carriers reset cleanly for reuse | FR-019 | Unit-test `GraphPrefetchState::reset(...)` and `LinearPrefetchState::reset(...)` so the staged D1 seam already matches `read_stream_reset()` and `amrescan` lifecycle expectations |
| TC-038 | EXPLAIN counter struct records and resets staged stats | FR-024 | Unit-test `TqExplainCounters` mutation helpers and reset behavior without touching `scan.rs` |
| TC-041 | EXPLAIN property emission stays pure and gated | FR-024 | Unit-test `TqExplainCounters::explain_properties()` plus the pure emission gate that requires the `tqvector` option, `IndexScan` node kind, and `ec_hnsw` access method |
| TC-045 | EXPLAIN output group contract stays explicit | FR-024 | Unit-test the pure `"TQVector Stats"` group metadata so the eventual hook opens and closes the expected EXPLAIN section |
| TC-042 | Cumulative statistics counters record and reset staged metrics | FR-025 | Unit-test `TqStatsCounters` mutation helpers and reset behavior without touching runtime pgstat wiring |
| TC-046 | Cumulative statistics summary computes derived rates | FR-025 | Unit-test pure FR-025 summary logic for `bootstrap_hit_rate` and `quantizer_cache_rate`, including zero-denominator handling |
| TC-039 | Metadata tree-height callback value matches max_level | FR-020 | Unit-test `metadata_tree_height_callback_value(max_level)` across edge cases, including `u8::MAX` |
| TC-044 | PG18 callback-named planner helpers preserve pure contracts | FR-020, FR-023 | Unit-test `amgettreeheight_callback_value`, `amtranslatestrategy_callback`, and `amtranslatecmptype_callback` so the PG18 callback seam matches the existing pure behavior |
| TC-040 | Strategy reverse mapping rejects non-LT CompareTypes | FR-023 | Unit-test `compare_type_to_strategy(...)` across `COMPARE_INVALID`, `COMPARE_EQ`, `COMPARE_LE`, `COMPARE_GE`, `COMPARE_GT`, `COMPARE_NE`, `COMPARE_OVERLAP`, and `COMPARE_CONTAINED_BY` |

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
| TC-112 | CREATE INDEX USING ec_hnsw succeeds | FR-008-AC-1 | Build index on 1000-row table |
| TC-113 | Index scan returns top-k results | FR-009-AC-1 | `ORDER BY <#> LIMIT 10`, assert 10 rows |
| TC-114 | EXPLAIN shows ec_hnsw index scan | FR-009-AC-3, FR-006-AC-2 | Check EXPLAIN output |
| TC-115 | DELETE + VACUUM removes from results | FR-010-AC-1 | Delete row, VACUUM, search, assert absent |
| TC-116 | DROP EXTENSION CASCADE cleans up | FR-012-AC-2 | Check pg_type, pg_operator, pg_am after drop |
| TC-117 | Metadata page readable after CREATE INDEX | FR-007-AC-1 | Read page 0, validate M and ef_construction |
| TC-118 | Concurrent INSERT + VACUUM no errors | FR-010-AC-3 | Run VACUUM and INSERT concurrently for 10s |
| TC-119 | Crash recovery: REINDEX after kill -9 | FR-011-AC-1 | Build index, kill backend, restart, REINDEX |
| TC-120 | GUC ef_search affects results | FR-009-AC-4 | Compare recall at ef_search=10 vs ef_search=200 |
| TC-121 | GUC ef_search is session-settable | FR-009-AC-5 | `SET ec_hnsw.ef_search = 200` succeeds |
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
| TC-133 | Insert-drift statistics are queryable | FR-016-AC-4 | Read `ec_hnsw_index_admin_snapshot(regclass)`, assert `total_live_nodes`, `inserted_since_rebuild`, and `insert_drift_fraction` stay consistent after inserts and duplicate coalescing |
| TC-142 | Planner integration snapshot exposes cross-lane blockers | FR-009, FR-020 | Read `ec_hnsw_planner_integration_snapshot(regclass)`, assert modeled planner-cost readiness is true while planner activation, ordered scan readiness, and PG18 readiness remain false with explicit blocker strings |
| TC-137 | Pure planner cost model stays gated behind ADR-011 | FR-020, FR-009 | Unit-test `estimate_planner_cost(...)` crossover and edge cases while `amcostestimate` still returns prohibitive costs |
| TC-138 | Cost snapshot exposes modeled and gated planner costs | FR-020, FR-009 | Read `ec_hnsw_index_cost_snapshot(...)`, assert modeled costs are finite, gated costs remain prohibitive, tuning/metadata inputs are explicit, and tree-height sourcing is reported as metadata fallback until PG18 callback wiring exists |

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

## PG18 Integration Tests (`cargo pgrx test --features pg18`)

| TC | Description | Traces | Method |
|---|---|---|---|
| TC-201 | Graph scan uses ReadStream on PG18 | FR-019-AC-1 | Instrument scan to verify read_stream_next_buffer is called instead of ReadBufferExtended during bootstrap |
| TC-202 | Linear scan uses ReadStream on PG18 | FR-019-AC-1 | Instrument scan to verify read_stream_next_buffer is called during linear scan fallback |
| TC-203 | PG17 fallback uses ReadBufferExtended | FR-019-AC-2 | Compile with `--features pg17`, verify no read_stream symbols linked |
| TC-204 | No buffer pin leaks after amendscan | FR-019-AC-4 | After scan completes, verify zero outstanding buffer pins via pg_buffercache |
| TC-205 | Vacuum tuple count uses streaming reads | FR-019-AC-5 | On PG18, verify count_element_tuples uses ReadStream |
| TC-206 | Planner selects ec_hnsw for ORDER BY LIMIT | FR-020-AC-1 | `EXPLAIN SELECT ... ORDER BY col <#> $q LIMIT 10` shows Index Scan on 10K-row table |
| TC-207 | Planner may prefer seqscan on small tables | FR-020-AC-2 | `EXPLAIN` on 50-row table, verify planner considers seqscan |
| TC-208 | Cost model reads metadata | FR-020-AC-3 | Create indexes with different m values, verify EXPLAIN costs differ |
| TC-209 | amgettreeheight returns max_level | FR-020-AC-4 | Build index, verify amgettreeheight returns expected level |
| TC-210 | Parallel build with 4 workers | FR-021-AC-1 | `SET max_parallel_maintenance_workers = 4; CREATE INDEX ...`, verify workers launched |
| TC-211 | Parallel build correctness | FR-021-AC-2 | Compare recall of parallel-built vs serial-built index on same data |
| TC-212 | Small table serial fallback | FR-021-AC-5 | 100-row table build does not launch workers |
| TC-213 | Vacuum removes dead heap TIDs | FR-022-AC-1, FR-022-AC-2 | DELETE + VACUUM + search, verify deleted row absent |
| TC-214 | Vacuum maintains graph connectivity | FR-022-AC-3 | Delete 10%, VACUUM, measure recall ≥ 80% of pre-vacuum |
| TC-215 | Vacuum concurrent safety | FR-022-AC-4 | `scripts/vacuum_concurrency_scratch.sh --duration 60` runs concurrent INSERT + ec_hnsw scan + VACUUM for 60s, then performs a final post-quiesce `VACUUM (ANALYZE)` and asserts the live index's reachable live-element count stays within 90% of a freshly rebuilt reference ec_hnsw index on the same final table data |
| TC-216 | Strategy translation: COMPARE_LT | FR-023-AC-2 | Verify amtranslatestrategy(1) returns COMPARE_LT |
| TC-217 | Strategy translation: invalid | FR-023-AC-4 | Verify amtranslatestrategy(99) returns COMPARE_INVALID |
| TC-218 | EXPLAIN (tqvector) recognized | FR-024-AC-1 | `EXPLAIN (tqvector) SELECT ...` parses without error |
| TC-219 | EXPLAIN (tqvector) shows stats | FR-024-AC-2 | Output includes Bootstrap Expansions, Elements Scored, etc. |
| TC-220 | EXPLAIN without tqvector option: no extra output | FR-024-AC-3 | Standard EXPLAIN does not show tqvector stats |
| TC-221 | EXPLAIN (tqvector, ANALYZE) shows actuals | FR-024-AC-4 | Non-zero counter values in output |
| TC-222 | tqvector_stats() returns counters | FR-025-AC-1 | SELECT * FROM tqvector_stats() returns row |
| TC-223 | Stats counters increment | FR-025-AC-2 | Run 10 scans, verify total_scans_started ≥ 10 |
| TC-224 | Stats reset works | FR-025-AC-3 | Reset, verify all counters zero |
| TC-225 | pg_get_loaded_modules shows tqvector | FR-026-AC-1 | Query pg_get_loaded_modules, verify name and version |
| TC-226 | Module version matches Cargo.toml | FR-026-AC-2 | Compare reported version to Cargo.toml version field |
| TC-227 | PG18 build succeeds | FR-027-AC-1 | `cargo pgrx build --features pg18 --release` exits 0 |
| TC-228 | PG17 build succeeds | FR-027-AC-2 | `cargo pgrx build --features pg17 --release` exits 0 |
| TC-229 | _PG_init registers PG18 diagnostics | FR-027-AC-4 | After CREATE EXTENSION, verify the EXPLAIN option is registered and the remaining pgstat-kind blocker is explicit |
| TC-230 | ADR-011 f64::MAX override removed | FR-020-AC-5 | Inspect source: no `f64::MAX` in cost.rs; ADR-011 status is SUPERSEDED |
| TC-231 | CREATE INDEX CONCURRENTLY with parallel workers | FR-021-AC-4 | `SET max_parallel_maintenance_workers = 2; CREATE INDEX CONCURRENTLY ... USING ec_hnsw ...` succeeds, index is usable |
| TC-232 | Parallel build uses GenericXLog | FR-021-AC-6 | Inspect source: all page writes in leader graph serialization use GenericXLogStart/Finish |
| TC-233 | Vacuum page writes use GenericXLog | FR-022-AC-5 | Inspect source: all page writes in ambulkdelete use GenericXLogStart/Finish. REINDEX after kill -9 during vacuum recovers cleanly |
| TC-234 | Vacuum updates pg_class.reltuples | FR-022-AC-6 | Delete 100 rows from 1000-row table, VACUUM, verify `SELECT reltuples FROM pg_class WHERE relname = 'idx'` reflects ~900 |
| TC-235 | Strategy translation callbacks registered | FR-023-AC-1 | On PG18, verify `amtranslatestrategy` and `amtranslatecmptype` are non-null in IndexAmRoutine via pg_am inspection |
| TC-236 | amtranslatecmptype reverse mapping | FR-023-AC-3 | Verify `amtranslatecmptype(COMPARE_LT)` returns strategy 1 |
| TC-237 | EXPLAIN hook chains with previous hook | FR-024-AC-5 | Install a dummy explain_per_node_hook before loading tqvector, verify both hooks fire on EXPLAIN |
| TC-238 | Stats persist within session | FR-025-AC-4 | Run 5 scans, read stats, run 5 more scans, verify counters accumulated (not reset between queries) |
| TC-239 | tqvector_stats() absent on PG17 | FR-025-AC-5 | Compile with `--features pg17`, verify `SELECT * FROM tqvector_stats()` raises ERROR or function does not exist |
| TC-240 | PG17 and PG18 tests both pass | FR-027-AC-3 | CI matrix: `cargo pgrx test pg17` and `cargo pgrx test pg18` both exit 0 |

## PG18 Benchmarks

| BC | Description | Traces | Target | Status |
|---|---|---|---|---|
| BC-032 | Cold-cache HNSW latency: io_method=sync vs worker vs io_uring | NFR-006, FR-019 | Report p50/p99 at effective_io_concurrency=0,4,8,16,32 | Blocked (PG18) |
| BC-033 | Cold-cache HNSW speedup: streaming vs sync baseline | NFR-006, FR-019 | ≥ 2x improvement at eic=16 | Blocked (PG18) |
| BC-034 | Warm-cache HNSW: streaming vs sync (regression check) | NFR-006, FR-019 | No regression (≤ 5% overhead) | Blocked (PG18) |
| BC-035 | Cold-cache linear scan: streaming vs sync | NFR-006, FR-019 | Report throughput improvement | Blocked (PG18) |
| BC-036 | Parallel build speedup: 1 vs 2 vs 4 workers (100K rows) | FR-021 | 4 workers ≤ 60% of serial time | Blocked (PG18) |
| BC-037 | Parallel build: parallel vs serial recall comparison | FR-021 | Within 1% recall difference | Blocked (PG18) |
| BC-038 | Vacuum impact on scan latency (50K, 10% deleted) | FR-022 | Report pre/post vacuum scan latency | Blocked (PG18) |

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
| FR-019 | TC-201, TC-202, TC-203, TC-204, TC-205, BC-032, BC-033, BC-034, BC-035 |
| FR-020 | TC-206, TC-207, TC-208, TC-209, TC-230 |
| FR-021 | TC-210, TC-211, TC-212, TC-231, TC-232, BC-036, BC-037 |
| FR-022 | TC-213, TC-214, TC-215, TC-233, TC-234, BC-038 |
| FR-023 | TC-216, TC-217, TC-235, TC-236 |
| FR-024 | TC-218, TC-219, TC-220, TC-221, TC-237 |
| FR-025 | TC-222, TC-223, TC-224, TC-238, TC-239 |
| FR-026 | TC-225, TC-226 |
| FR-027 | TC-227, TC-228, TC-229, TC-240 |
| NFR-006 | BC-032, BC-033, BC-034, BC-035 |
