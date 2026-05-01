---
artifact_type: test-matrix
name: ecaz
status: PARTIAL
---
# Test Matrix

This matrix follows the `/spec-matrix` skill shape. It replaces the stale HNSW-era inventory with current traceability for the multi-AM Ecaz implementation.

## Test Matrix Rules

1. Coverage: every acceptance criterion should trace to at least one test case or documented gap.
2. Option permutation: all valid reloption/GUC/storage-format combinations should be covered where they are normative.
3. Constraint boundary: min, max, below-min, and above-max boundaries should be covered for normative constraints.
4. Error path: documented error conditions should have tests.
5. State transition: build, scan, insert, vacuum, drop, and rebuild transitions should have tests.
6. Edge case: empty indexes, duplicate rows, non-finite data, dimension mismatch, storage-format mismatch, and deferred hardware gates should be explicit.

## Requirements Traceability

### Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR/NFR | Test/Validation | Coverage Status |
| --- | --- | --- | --- |
| StR-001 | US-001..US-005, FR-001..FR-018, FR-028..FR-030 | TC-001, TC-002, TC-003, TC-004, TC-013, TC-015 | Partial: legacy HNSW targets need updated product benchmark evidence |
| StR-002 | US-004, NFR-004, NFR-005 | TC-013, TC-014 | Partial: license tooling is documented but not run by default in this docs checkpoint |
| StR-003 | US-003, US-005, FR-008..FR-010, FR-030 | TC-004 | Partial: partition-specific evidence should be refreshed when next HNSW benchmark packet is opened |
| StR-004 | US-006..US-011, FR-019..FR-027, FR-030 | TC-005, TC-006, TC-017 | Partial: ReadStream/product speedup measurements remain deferred |
| StR-005 | US-012..US-014, FR-028..FR-036 | TC-002, TC-003, TC-004, TC-007..TC-012 | Complete for local implementation surface; product scale evidence deferred |
| StR-006 | US-015, US-016, FR-037, NFR-007..NFR-009 | TC-015, TC-016, TC-019 | Partial: product hardware gates are explicit gaps |

### User Story Coverage

| User Story | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| US-001 | Existing `tqvector` storage ACs | TC-001 | Complete for artifact/debug surface |
| US-002 | SQL nearest-neighbor query | TC-004, TC-007, TC-010 | Complete for local AM smoke/behavior tests |
| US-003 | Build HNSW index | TC-004, TC-006 | Partial: larger parallel build speedups deferred |
| US-004 | Extension lifecycle | TC-003, TC-014 | Complete for catalog/build surface |
| US-005 | HNSW vacuum cleanup | TC-004 | Partial: product recall-after-vacuum measurements deferred |
| US-006 | Async I/O scan | TC-005, TC-017 | Partial: live surface exists; cold-cache speedup measurement deferred |
| US-007 | Planner-visible cost model | TC-005 | Complete for local callback/cost behavior |
| US-008 | Parallel index build | TC-006, TC-016 | Partial: local implementation landed; AWS/RDS scale evidence deferred |
| US-009 | EXPLAIN diagnostics | TC-005, TC-008 | Complete for HNSW/IVF local diagnostics |
| US-010 | Vacuum removes deleted vectors | TC-004, TC-009, TC-012 | Complete for local AM behavior tests |
| US-011 | Operational statistics | TC-005 | Partial: reset for custom kind remains blocked upstream/local PG18 tree |
| US-012 | US-012-AC-1..3 | TC-002, TC-003, TC-004, TC-007, TC-010 | Complete for current SQL surface |
| US-013 | US-013-AC-1..3 | TC-007, TC-008, TC-009, TC-015 | Complete for local IVF v1; product claims deferred |
| US-014 | US-014-AC-1..3 | TC-010, TC-011, TC-012, TC-015 | Complete for local DiskANN v1; product claims deferred |
| US-015 | US-015-AC-1..3 | TC-015, TC-016 | Partial: product benchmark claim lane is a planned gate |
| US-016 | US-016-AC-1..3 | TC-019 | Complete for docs/spec traceability; command execution tests run on demand |

### Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001..FR-006 | Type, I/O, encode, scoring, operators | TC-001, TC-002, TC-003 | Complete for current unit/pg_test coverage |
| FR-007..FR-018 | HNSW layout/build/scan/vacuum/WAL/insert/scoring | TC-001, TC-004, TC-013 | Partial: old HNSW product benchmark rows need refreshed evidence |
| FR-019 | ReadStream integration | TC-005, TC-017 | Partial: behavior coverage exists; speedup evidence deferred |
| FR-020 | Planner cost estimation | TC-005 | Complete for local modeled/live cost surface |
| FR-021 | Parallel index build | TC-006, TC-016 | Partial: scale measurement deferred |
| FR-022 | Vacuum implementation | TC-004, TC-009, TC-012 | Complete for local behavior surfaces |
| FR-023 | Strategy translation callbacks | TC-005, TC-008 | Complete for PG18 callback coverage |
| FR-024 | Custom EXPLAIN | TC-005, TC-008 | Complete for HNSW/IVF local diagnostics |
| FR-025 | Custom statistics | TC-005 | Partial: shared reset path remains a known blocker |
| FR-026 | PG18 module identity | TC-005, TC-014 | Complete for PG18 build surface |
| FR-027 | pgrx PG18 support | TC-014 | Complete for current build configuration |
| FR-028 | FR-028-AC-1..4 | TC-002, TC-003 | Complete for canonical `ecvector` surface |
| FR-029 | FR-029-AC-1..3 | TC-003 | Complete for SQL bootstrap surface |
| FR-030 | FR-030-AC-1..4 | TC-004, TC-005, TC-006 | Partial: large-build measurement deferred |
| FR-031 | FR-031-AC-1..3 | TC-007 | Complete for local IVF build/storage behavior |
| FR-032 | FR-032-AC-1..3 | TC-008 | Complete for local IVF scan/rerank/cost behavior |
| FR-033 | FR-033-AC-1..3 | TC-009 | Complete for local IVF insert/vacuum/admin behavior |
| FR-034 | FR-034-AC-1..3 | TC-010 | Complete for local DiskANN build/storage behavior |
| FR-035 | FR-035-AC-1..3 | TC-011 | Complete for local DiskANN scan/prefilter/rerank behavior |
| FR-036 | FR-036-AC-1..3 | TC-012 | Complete for local DiskANN insert/vacuum/diagnostics behavior |
| FR-037 | FR-037-AC-1..4 | TC-019 | Complete for docs/spec traceability; CLI unit execution not run in this docs checkpoint |

### Non-Functional Requirement Coverage

| NFR | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | SQL latency benchmarks | TC-015, TC-016 | Partial: local rows exist; product latency claims need controlled hardware |
| NFR-002 | Storage-size measurement | TC-015 | Partial: local HNSW/IVF/DiskANN rows exist; full product accounting deferred |
| NFR-003 | Recall measurement | TC-015 | Partial: local recall rows exist; product claim gate deferred |
| NFR-004 | Safety, WAL, fuzz/inspection | TC-013 | Partial: static docs checkpoint did not run tests |
| NFR-005 | Build and CI | TC-014 | Partial: static docs checkpoint did not run build/test commands |
| NFR-006 | Async I/O cold-cache performance | TC-017 | Gap: measurement deferred |
| NFR-007 | Benchmark provenance | TC-015 | Complete for current docs/review-packet citations |
| NFR-008 | Scale boundary | TC-016 | Complete as policy; execution deferred |
| NFR-009 | CLI drift and artifact discipline | TC-019 | Complete for docs/spec traceability; command-tree execution audit deferred to CLI tests |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | `tqvector` artifact layout, I/O, encode, scoring | Unit / pg_test | P0 | FR-001..FR-006, FR-013..FR-018 | Implemented |
| TC-002 | `ecvector` typmod, I/O, casts, encode defaults | Unit / pg_test | P0 | US-012, FR-028 | Implemented |
| TC-003 | SQL bootstrap registers extension objects | pg_test / catalog inspection | P0 | US-004, FR-012, FR-029 | Implemented |
| TC-004 | HNSW build, scan, insert, vacuum, storage formats | pg_test | P0 | US-002, US-003, US-005, FR-007..FR-018, FR-030 | Implemented |
| TC-005 | PG18 planner, EXPLAIN, stats, module identity | pg_test / inspection | P0 | US-006, US-007, US-009, US-011, FR-019..FR-027, FR-030 | Partial: custom stats reset blocked |
| TC-006 | HNSW parallel build and DSM graph assembly | pg_test / benchmark | P1 | US-008, FR-021, FR-030 | Partial: local implementation landed, scale evidence deferred |
| TC-007 | IVF build, reloptions, metadata, storage formats | pg_test | P0 | US-013, FR-031 | Implemented |
| TC-008 | IVF scan, GUC overrides, rerank, cost, EXPLAIN | pg_test | P0 | US-013, FR-032 | Implemented |
| TC-009 | IVF insert, vacuum, admin/drift snapshots | pg_test | P0 | US-013, FR-033 | Implemented |
| TC-010 | DiskANN build, unit-normalized contract, graph storage | pg_test | P0 | US-014, FR-034 | Implemented |
| TC-011 | DiskANN scan, prefilter, list-size override, rerank | pg_test | P0 | US-014, FR-035 | Implemented |
| TC-012 | DiskANN insert, vacuum repair, diagnostics | pg_test | P0 | US-014, FR-036 | Implemented |
| TC-013 | Safety, WAL discipline, unsafe/fuzz/license review | Unit / fuzz / inspection | P1 | NFR-004, FR-011 | Partial: run explicitly when risk warrants |
| TC-014 | PG18 primary and PG17 compatibility builds | CI / build | P0 | US-004, FR-026, FR-027, NFR-005 | Partial: not run in this docs checkpoint |
| TC-015 | Local benchmark provenance for HNSW/IVF/DiskANN | Review packet / docs audit | P1 | US-015, NFR-001, NFR-002, NFR-003, NFR-007 | Implemented for current docs |
| TC-016 | AWS/RDS-class product benchmark gate | Benchmark | P2 | NFR-008, US-015 | Gap: deferred |
| TC-017 | ReadStream cold-cache speedup gate | Benchmark | P2 | NFR-006, FR-019 | Gap: deferred |
| TC-018 | HNSW insert decontention follow-up | Benchmark / implementation | P2 | Future Task 13 | Gap: future work |
| TC-019 | `ecaz` CLI command tree, profiles, logging, and docs links | Unit / docs audit | P1 | US-016, FR-037, NFR-009 | Implemented for docs traceability; CLI tests run on demand |

## Option Permutation Matrix

| Test Case | Option Set | Required Coverage | Expected Behavior |
| --- | --- | --- | --- |
| TC-004 | `ec_hnsw.storage_format` | `auto`, TurboQuant/PQ-FastScan-family formats supported by current code | Valid formats build/scan; incompatible live storage-format changes reject until rebuild |
| TC-004 | `ec_hnsw.ef_search` | relation default, session override, reset | Effective scan breadth follows session override when set |
| TC-006 | `ec_hnsw.enable_parallel_build_concurrent_dsm` | true, false | true uses concurrent DSM path when eligible; false uses diagnostic fallback |
| TC-007 | `ec_ivf.storage_format` | `auto`, `turboquant`, `pq_fastscan`, `rabitq` | Valid formats build; invalid strings reject |
| TC-007 | `ec_ivf.rerank` | `auto`, `off`, `heap_f32`, `source_column` | First three supported; `source_column` rejected in v1 |
| TC-008 | `ec_ivf.nprobe`, `ec_ivf.rerank_width` | relation, session, auto | Effective scan settings report correct source |
| TC-010 | `ec_diskann.storage_format` | `pq_fastscan` | Valid; other values reject |
| TC-011 | `ec_diskann.prefilter_kind` | `auto`, `binary_sidecar`, `grouped_pq` | Selects persisted sidecar or grouped-PQ fallback as requested |
| TC-011 | `ec_diskann.list_size` | relation, session override, reset | Effective scan breadth reports correct source |
| TC-019 | `ecaz` command groups | `corpus`, `bench`, `compare`, `dev`, `quant`, `stress` | Help tree, README tree, and dispatch modules stay aligned |
| TC-019 | `ecaz` AM profiles | `ec_hnsw`, `ec_ivf`, `ec_diskann` | Profile metadata selects AM, opclass, embedding type, scan GUC, sweep axis, and reloption set |
| TC-019 | `ecaz` logging | terminal output, `--log-file`, dev SQL `--log-output` | Review evidence can be stored under packet-local artifacts |

## Constraint Boundary Tests

| Constraint | Boundary Type | Test Value | Test Case | Expected |
| --- | --- | --- | --- | --- |
| `ecvector(N)` dimension | Exact | N values | TC-002 | Pass |
| `ecvector(N)` dimension | Mismatch | N-1 / N+1 values | TC-002 | ERROR |
| `encode_to_ecvector` defaults | Canonical | `(4, 42)` | TC-002 | Pass |
| `encode_to_ecvector` defaults | Non-canonical | any other bits/seed | TC-002 | ERROR |
| HNSW reloptions | Min/max and outside range | `m`, `ef_construction`, `ef_search` | TC-004 | Boundary pass, outside ERROR |
| IVF reloptions | Min/max and outside range | `nlists`, `nprobe`, `rerank_width`, `pq_group_size` | TC-007, TC-008 | Boundary pass, outside ERROR |
| DiskANN reloptions | Min/max and outside range | `graph_degree`, `build_list_size`, `list_size`, `rerank_budget`, `top_k`, `alpha` | TC-010, TC-011 | Boundary pass, outside ERROR |
| DiskANN unit norm | Within epsilon | `||v|| ~= 1.0` | TC-010 | Pass |
| DiskANN unit norm | Outside epsilon / non-finite | invalid norms | TC-010 | ERROR or warning by context |

## Edge Cases

| ID | Description | Related Req | Test Case | Risk if Untested |
| --- | --- | --- | --- | --- |
| EC-001 | Empty indexes and repeated rescans | FR-030, FR-032, FR-035 | TC-004, TC-008, TC-011 | Executor may emit stale state or crash |
| EC-002 | Duplicate vectors and duplicate heap TID overflow | FR-030, FR-036 | TC-004, TC-012 | Missing rows or corrupted duplicate chains |
| EC-003 | Non-finite fp32 input | FR-028, FR-034 | TC-002, TC-010 | Invalid scores or backend errors |
| EC-004 | Storage-format switch without rebuild | FR-030, FR-031 | TC-004, TC-007 | Incorrect decoding of persisted index pages |
| EC-005 | Dead tuple cleanup during vacuum | FR-022, FR-033, FR-036 | TC-004, TC-009, TC-012 | Deleted rows returned or graph connectivity loss |
| EC-006 | Product benchmark claim without controlled hardware | NFR-007, NFR-008 | TC-016 | Misleading docs or unsupported roadmap decisions |
| EC-007 | CLI README command tree drifts from Clap tree | FR-037, NFR-009 | TC-019 | Operators run stale commands or miss supported workflows |

## Integration Test Matrix

Ecaz has one required local service integration: PostgreSQL itself.

| Integration ID | Purpose | Service | Type | Test Cases | Status |
| --- | --- | --- | --- | --- | --- |
| INT-001 | Extension lifecycle and catalog registration | PostgreSQL 18 | database | TC-003, TC-005, TC-014 | Partial: not run in this docs checkpoint |
| INT-002 | PG17 fallback build/test lane | PostgreSQL 17 | database | TC-014 | Partial: run on demand |
| INT-003 | Real-corpus benchmark surfaces | PostgreSQL plus local corpus files | database/filesystem | TC-015, TC-016, TC-017 | Partial: local evidence exists, product gates deferred |
| INT-004 | CLI operator benchmark and stress workflows | PostgreSQL plus local corpus files | database/filesystem | TC-019 | Partial: docs/spec trace complete; execution run on demand |

## Coverage Gaps

| Gap ID | Description | Risk Level | Mitigation |
| --- | --- | --- | --- |
| GAP-001 | Product benchmark claims for IVF/DiskANN on controlled AWS/RDS-class hardware | Medium | Keep docs labeled local; open dedicated measurement packet before product claims |
| GAP-002 | ReadStream cold-cache speedup verification | Medium | Run PG18 cold-cache matrix when hardware setup is ready |
| GAP-003 | Custom pgstat reset support | Low | Track upstream/local PG18 support for custom-kind reset |
| GAP-004 | HNSW insert throughput decontention | Medium | Track as future Task 13 work |
| GAP-005 | Full requirement-to-individual-test function inventory | Low | Generate from source/test names if a stricter audit packet is needed |
| GAP-006 | Automated CLI README-vs-Clap tree drift check | Low | Add a generated help snapshot or parser-backed docs check if the CLI surface starts changing frequently |

## Test Execution Summary

This checkpoint is a docs/spec cleanup. Tests were not run by default under the repository checkpoint policy.

| Category | Total Groups | Implemented / Evidenced | Partial | Gap |
| --- | ---: | ---: | ---: | ---: |
| Unit / pg_test behavior groups | 15 | 12 | 3 | 0 |
| Benchmark / measurement groups | 4 | 1 | 1 | 2 |
| Integration groups | 4 | 0 | 4 | 0 |
