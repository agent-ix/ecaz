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

## Coverage Audit Baseline

The matrix is a correctness baseline, not only a generated inventory. "Complete"
requires evidence that the cited tests assert the relevant behavior, not merely
that a similarly named test exists.

| Source | Baseline Evidence | Current Interpretation |
| --- | --- | --- |
| Rust unit and pg_test inventory | SPIRE has broad pure-Rust and PG18 fixture coverage across assignment, metadata, storage, options, scan, coordinator, CustomScan, DML, update, vacuum, and diagnostics modules. | Strong implementation baseline; still requires AC-level trace rows for any "Complete" claim. |
| `review/31070-spire-phase12c-coverage-audit` | Independent audit inventoried SPIRE tests, read representative tests for assertion quality, and identified weak or indirect areas such as CustomScan lifecycle, Stage E live coverage, matrix string-only tests, DML row-state assertions, and operator surface coverage. | Treat SPIRE distributed and CustomScan coverage as `Partial` until the named weak areas are closed or explicitly accepted as gaps. |
| `plan/tasks/34-comprehensive-hardening.md`, `docs/hardening.md`, and `review/30034-task34-comprehensive-hardening` | Task 34 documented hardening lanes for supply chain, unsafe/static hygiene, Miri, cargo-careful, fuzzing, Kani, Flux, Loom, Shuttle, sanitizers, SQLsmith, Rudra, MIRAI, and aggregate local/nightly targets. Packet-local raw logs in `review/30034` currently support only the installer, MIRAI, Flux, and Rudra-family evidence that appears in that packet manifest. | Adds `TC-034` as the hardening evidence lane for `NFR-004`, but unpacketed local aggregate, sanitizer, fuzz, cargo-careful, Kani, Loom, Shuttle, cargo-vet, cargo-geiger, and AFL claims remain explicit evidence gaps until logs are packeted. Live PG18 sanitizer and SQLsmith lanes remain manually gated gaps. |
| Benchmark reporting standard | `NFR-015` defines identity fields and metric families across AMs, quantizers, storage formats, option sets, and product evidence classes. | The standard itself is implemented, but benchmark rows are only complete after each packet conforms row-by-row. |

## Analysis Requirement Coverage Rules

Analysis requirements are verified by evidence artifacts, not by source-code
presence alone.

| Rule | General Requirement | Ecaz Application |
| --- | --- | --- |
| AR-1 Risk class | Name the risk class under analysis. | `NFR-004` separates supply-chain, unsafe/static hygiene, pure-Rust UB, parser/decoder fuzzing, bounded proofs, concurrency, and live PostgreSQL callback safety. |
| AR-2 Evidence command | Name the command or method that produces evidence. | Task 34 maps lanes to Make/script commands such as `make hardening-local`, `make fuzz-all-short`, `make kani`, `make loom`, and `make sqlsmith-pg18`. |
| AR-3 Gate level | State whether the lane is PR, nightly, weekly/manual, local-only, or report-only. | `docs/hardening.md` distinguishes local aggregates, nightly/toolchain-sensitive lanes, standalone reports, and live PG18/manual lanes. |
| AR-4 Artifact | Identify the durable artifact. | Review packets store raw tool logs and `artifacts/manifest.md`; local-only runs must be packeted before cited as evidence. |
| AR-5 Interpretation | State pass/fail, skip, or triage behavior. | Unsupported sanitizer lanes may skip with an explicit platform message; Rudra/MIRAI/Flux findings are report-only until triaged. |
| AR-6 Model boundary | State what the tool cannot prove. | Miri/Kani/cargo-careful cover pure Rust helpers; pg_test, PG sanitizers, and SQLsmith cover pgrx/SPI/libpq/live executor boundaries. |

## Requirements Traceability

### Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR/NFR | Test/Validation | Coverage Status |
| --- | --- | --- | --- |
| StR-001 | US-001..US-005, FR-001..FR-018, FR-028..FR-030 | TC-001, TC-002, TC-003, TC-004, TC-013, TC-015 | Partial: legacy HNSW targets need updated product benchmark evidence |
| StR-002 | US-004, NFR-004, NFR-005 | TC-013, TC-014, TC-034 | Partial: Task 34 hardening docs exist, but only packet-local Task 34 logs count as completed evidence; unpacketed local lanes plus PG18 live sanitizer and SQLsmith lanes remain gaps |
| StR-003 | US-003, US-005, FR-008..FR-010, FR-030 | TC-004 | Partial: partition-specific evidence should be refreshed when next HNSW benchmark packet is opened |
| StR-004 | US-006..US-011, FR-019..FR-027, FR-030 | TC-005, TC-006, TC-017 | Partial: ReadStream/product speedup measurements remain deferred |
| StR-005 | US-012..US-014, FR-028..FR-036 | TC-002, TC-003, TC-004, TC-007..TC-012 | Complete for local implementation surface; product scale evidence deferred |
| StR-005 SPIRE extension | US-018..US-020, US-022, FR-048..FR-060, NFR-013, NFR-014 | TC-020 SPIRE, TC-021..TC-025, TC-034 | Partial: implementation baseline is broad, but AC-level mapping, CustomScan lifecycle proof, Stage E live coverage, and product-scale AWS evidence remain gaps |
| StR-006 | US-015, US-016, US-017 benchmark suites, FR-037, FR-038 benchmark suites, NFR-007..NFR-009, NFR-015 | TC-015, TC-016, TC-019, TC-020 benchmark suites, TC-033 | Partial: product hardware gates are explicit gaps |
| StR-007 cloud | US-021, FR-044..FR-047, NFR-010, NFR-011 | TC-026..TC-032 | Planned: cloud harness implementation begins on `feat/cloud-test-harness` |

### User Story Coverage

| User Story | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| US-001 | US-001-AC-1..4 | TC-001 | Complete for artifact/debug behavior group; strict per-AC evidence inventory remains `GAP-018` |
| US-002 | US-002-AC-1..4 | TC-004, TC-007, TC-010 | Complete for local AM smoke/behavior group; strict per-AC evidence inventory remains `GAP-018` |
| US-003 | US-003-AC-1..4 | TC-004, TC-006 | Partial: larger parallel build speedups deferred; strict per-AC evidence inventory remains `GAP-018` |
| US-004 | US-004-AC-1..4 | TC-003, TC-014 | Complete for catalog/build behavior group; strict per-AC evidence inventory remains `GAP-018` |
| US-005 | US-005-AC-1..3 | TC-004 | Partial: product recall-after-vacuum measurements deferred; strict per-AC evidence inventory remains `GAP-018` |
| US-006 | US-006-AC-1..5 | TC-005, TC-017 | Partial: live surface exists; cold-cache speedup measurement deferred |
| US-007 | US-007-AC-1..4 | TC-005 | Complete for local callback/cost behavior group; strict per-AC evidence inventory remains `GAP-018` |
| US-008 | US-008-AC-1..4 | TC-006, TC-016 | Partial: local implementation landed; AWS/RDS scale evidence deferred |
| US-009 | US-009-AC-1..4 | TC-005, TC-008 | Complete for HNSW/IVF local diagnostics group; strict per-AC evidence inventory remains `GAP-018` |
| US-010 | US-010-AC-1..4 | TC-004, TC-009, TC-012 | Complete for local AM behavior group; strict per-AC evidence inventory remains `GAP-018` |
| US-011 | US-011-AC-1..4 | TC-005 | Partial: reset for custom kind remains blocked upstream/local PG18 tree |
| US-012 | US-012-AC-1..3 | TC-002, TC-003, TC-004, TC-007, TC-010 | Complete for current SQL surface |
| US-013 | US-013-AC-1..3 | TC-007, TC-008, TC-009, TC-015 | Complete for local IVF v1; product claims deferred |
| US-014 | US-014-AC-1..3 | TC-010, TC-011, TC-012, TC-015 | Complete for local DiskANN v1; product claims deferred |
| US-015 | US-015-AC-1..4 | TC-015, TC-016, TC-033 | Partial: product benchmark claim lane is a planned gate |
| US-016 | US-016-AC-1..3 | TC-019 | Complete for docs/spec traceability; command execution tests run on demand |
| US-017 benchmark suites | US-017-AC-1..5 | TC-020 benchmark suites, TC-033 | Complete for first auto-runner surface; tags/resume/results extraction implemented, richer thresholds deferred |
| US-018 | US-018-AC-1..6 | TC-021, TC-022, TC-023 | Implemented for relation-backed local stores, PID hash placement, store diagnostics, strict/degraded handling, and sequential backend read scheduling; true parallel local-store execution deferred |
| US-019 | US-019-AC-1..6 | TC-023, TC-024 | Implemented for CustomScan distributed reads, placement-aware dispatch, typed remote tuple payloads, and origin-node visibility; AWS product evidence deferred |
| US-020 | US-020-AC-1..6 | TC-023, TC-025 | Implemented for epoch publication, delta/replacement maintenance, split/merge/vacuum hooks, diagnostics, and coordinator DML/2PC recovery; background prepared-xact recovery deferred |
| US-022 | US-022-AC-1..6 | TC-020 SPIRE, TC-021, TC-022, TC-025 | Implemented for local build/publish/search lifecycle and operator-visible maintenance; long-running scale evidence deferred |
| US-021 | US-021-AC-1..5 | TC-026, TC-029, TC-030 | Planned: implementation in progress on cloud branch |

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
| FR-029 | FR-029-AC-1..4 | TC-003 | Complete for SQL bootstrap surface, including `ec_spire` registration |
| FR-030 | FR-030-AC-1..4 | TC-004, TC-005, TC-006 | Partial: large-build measurement deferred |
| FR-031 | FR-031-AC-1..3 | TC-007 | Complete for local IVF build/storage behavior |
| FR-032 | FR-032-AC-1..3 | TC-008 | Complete for local IVF scan/rerank/cost behavior |
| FR-033 | FR-033-AC-1..3 | TC-009 | Complete for local IVF insert/vacuum/admin behavior |
| FR-034 | FR-034-AC-1..3 | TC-010 | Complete for local DiskANN build/storage behavior |
| FR-035 | FR-035-AC-1..3 | TC-011 | Complete for local DiskANN scan/prefilter/rerank behavior |
| FR-036 | FR-036-AC-1..3 | TC-012 | Complete for local DiskANN insert/vacuum/diagnostics behavior |
| FR-037 | FR-037-AC-1..4 | TC-019 | Complete for docs/spec traceability; CLI unit execution not run in this docs checkpoint |
| FR-038 benchmark suites | FR-038-AC-1..8 | TC-020 benchmark suites, TC-033 | Complete for first auto-runner surface; full schema-driven report generation remains iterative |
| FR-039..FR-043 tombstones | No active ACs | Spec inspection | Superseded: retained tombstone files preserve immutable ID history and point to `FR-048..FR-060` replacements |
| FR-048 | FR-048-AC-1..8 | TC-020 SPIRE, TC-021, TC-024, TC-025 | Complete for domain model, identities, epochs, placement, and read/write boundary definitions |
| FR-049 | FR-049-AC-1..3 | TC-020 SPIRE, TC-022, TC-034 | Partial: common header decode and rejection paths exist; external fixture compatibility remains a gap before format freeze |
| FR-050 | FR-050-AC-1..3 | TC-020 SPIRE, TC-022, TC-034 | Partial: Leaf V2 round-trips and invariants exist; byte-for-byte compatibility fixtures remain a gap before format freeze |
| FR-051 | FR-051-AC-1..3 | TC-020 SPIRE, TC-022, TC-034 | Partial: routing/delta/top-graph structure is covered; graph/topology malformed-payload fixture coverage should be pinned per AC |
| FR-052 | FR-052 ACs | TC-020 SPIRE, TC-021, TC-023 | Partial: build/publish implementation baseline exists; long-running publish failure and retention stress evidence deferred |
| FR-053 | FR-053 ACs | TC-021, TC-022, TC-023 | Partial: eager local scan and routing evidence exists; parametric candidate-budget and multi-store scale coverage remain gaps |
| FR-054 | FR-054 ACs | TC-023, TC-025 | Partial: local maintenance coverage exists; full split/merge/vacuum lifecycle stress remains a gap |
| FR-055 | FR-055 ACs | TC-021, TC-024 | Partial: placement and topology readiness are covered; multi-remote cardinality and AWS topology evidence deferred |
| FR-056 | FR-056 ACs | TC-024, TC-034 | Partial: typed transport baseline exists; PG18 live type matrix and schema-drift failure coverage require row-level evidence |
| FR-057 | FR-057 ACs | TC-023, TC-024, TC-034 | Partial: executor fault/readiness matrix exists; Stage E live coverage is not complete for every fault category |
| FR-058 | FR-058 ACs | TC-024, TC-034 | Partial: CustomScan distributed reads are implemented; lifecycle callback, mark/restore exclusion, and rescan/end-after-cancel proof remain gaps |
| FR-059 | FR-059-AC-1..9 | TC-025, TC-034 | Partial: coordinator-routed DML baseline exists; repeated remote prepared-branch uniqueness and row-state assertions need direct evidence |
| FR-060 | FR-060-AC-1..8 | TC-021, TC-023, TC-024, TC-025 | Partial: diagnostics and fail-closed reporting exist; operator surface drift checks and matrix-to-live coverage remain gaps |
| FR-044 | FR-044-AC-1..4 | TC-026, TC-030 | Planned: cloud command surface and idempotence |
| FR-045 | FR-045-AC-1..4 | TC-027 | Planned: terraform module and profile selection |
| FR-046 | FR-046-AC-1..3 | TC-028 | Planned: dataset registry and parquet staging |
| FR-047 | FR-047-AC-1..4 | TC-029, TC-032 | Planned: in-VPC parallel corpus load |

### Non-Functional Requirement Coverage

| NFR | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | SQL latency benchmarks | TC-015, TC-016 | Partial: local rows exist; product latency claims need controlled hardware |
| NFR-002 | Storage-size measurement | TC-015 | Partial: local HNSW/IVF/DiskANN rows exist; full product accounting deferred |
| NFR-003 | Recall measurement | TC-015 | Partial: local recall rows exist; product claim gate deferred |
| NFR-004 | NFR-004-AC-1..5 | TC-013, TC-034 | Partial: Task 34 documents the lane surface, but only packet-local raw logs count as completed evidence; unpacketed local lanes, PG18 sanitizer, and SQLsmith remain gaps |
| NFR-005 | Build and CI | TC-014 | Partial: static docs checkpoint did not run build/test commands |
| NFR-006 | Async I/O cold-cache performance | TC-017 | Gap: measurement deferred |
| NFR-007 | Benchmark provenance | TC-015 | Complete for current docs/review-packet citations |
| NFR-008 | Scale boundary | TC-016 | Complete as policy; execution deferred |
| NFR-009 | CLI drift and artifact discipline | TC-019 | Complete for docs/spec traceability; command-tree execution audit deferred to CLI tests |
| NFR-010 | Cloud cost discipline (status reporting, no NAT, --confirm-cost gate) | TC-031 | Planned: cloud harness implementation in progress |
| NFR-011 | Cloud corpus load throughput targets | TC-032 | Planned: targets baseline once first `1m` run lands |
| NFR-012 | Cloud throughput targets | TC-016, TC-032 | Partial: targets are specified; product evidence is gated on controlled cloud runs |
| NFR-013 | SPIRE local readiness and capacity | TC-020 SPIRE, TC-021, TC-022, TC-023, TC-025 | Partial: implementation traceability exists; full capacity envelope needs controlled local storage evidence |
| NFR-014 | SPIRE transport security and operations | TC-024, TC-025 | Partial: v1 contract specifies TLS, timeout, cancellation, and observability behavior; deployment evidence deferred |
| NFR-015 | Benchmark reporting standard | TC-033 | Partial: standard is specified; existing and future benchmark rows must conform packet-by-packet before being marked complete |

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
| TC-020 benchmark suites | `ecaz bench suite` dry-run, execution manifest, audit, status, report, and results extraction | Unit / CLI smoke | P1 | US-017 benchmark suites, FR-038 benchmark suites, NFR-007, NFR-009 | Implemented for first auto-runner surface |
| TC-020 SPIRE | SPIRE partition-object domain model and binary storage formats | Design packet / pg_test | P0 | US-022, FR-048, FR-049, FR-050, FR-051, FR-052, NFR-013 | Implemented for spec traceability; format-freeze binary compatibility tests should be added before external persistence commitments |
| TC-021 | SPIRE local store configuration, placement, and diagnostics | SQL / pg_test | P1 | US-018, US-022, FR-053, FR-055, FR-060, NFR-013 | Implemented for local v1 behavior; true parallel local-store execution deferred |
| TC-022 | SPIRE routing, scoring, dedupe, and heap visibility handling | pg_test | P0 | US-018, US-022, FR-050, FR-051, FR-053 | Implemented for eager bounded local scans |
| TC-023 | SPIRE epoch consistency, degraded mode, retention, and failed publish | pg_test / fault injection | P0 | US-018, US-019, US-020, FR-052, FR-054, FR-057, FR-060, NFR-013 | Implemented for strict/degraded v1 paths; long retention stress evidence deferred |
| TC-024 | SPIRE distributed CustomScan and typed remote transport | Integration / pg_test | P0 | US-019, FR-055, FR-056, FR-057, FR-058, FR-060, NFR-014 | Implemented for PostgreSQL-node readiness; AWS/RDS product evidence deferred |
| TC-025 | SPIRE DML, split/merge, vacuum, replacement, 2PC recovery, and cleanup lifecycle | pg_test / stress | P1 | US-020, US-022, FR-054, FR-059, FR-060 | Implemented for v1 contract; background prepared-xact recovery and cross-shard embedding moves deferred |
| TC-026 | `ecaz cloud` lifecycle (up/install/down/status) idempotence and JSON status | Integration / CLI smoke | P0 | US-021, FR-044 | Planned: implementation in progress |
| TC-027 | Terraform module plans clean for every profile, no NAT, no SSH | Static / `terraform plan` | P0 | FR-045, NFR-010 | Planned: implementation in progress |
| TC-028 | Cloud dataset registry coverage, parquet staging SHA verification, BIGANN adapter | Unit / staging dry-run | P1 | FR-046 | Planned: implementation in progress |
| TC-029 | In-VPC parallel corpus load, row-count match, `--resume` correctness | Integration / SSM exec | P0 | US-021, FR-047 | Planned: implementation in progress |
| TC-030 | Pause/resume preserves data; snapshot + `--from-snapshot` skips re-load | Integration | P1 | US-021, FR-044, NFR-010 | Planned: implementation in progress |
| TC-031 | `--confirm-cost` gate, status `$/hr` and `$/mo` reporting, S3 lifecycle rule | Unit / static | P1 | NFR-010 | Planned: implementation in progress |
| TC-032 | Corpus load throughput meets per-profile NFR-011 targets | Benchmark | P1 | NFR-011, FR-047 | Planned: baseline once first `1m` run lands |
| TC-033 | Benchmark reporting standard docs/spec audit | Docs / spec audit | P1 | US-015, US-017, FR-038, NFR-015 | Implemented for the standard; future benchmark packets apply it row-by-row |
| TC-034 | Task 34 hardening and analysis lanes | Static analysis / fuzz / model checking / sanitizer / supply-chain audit | P0 | NFR-004, FR-011, FR-049, FR-050, FR-051, FR-052, FR-053, FR-054, FR-055, FR-056, FR-057, FR-058, FR-059 | Partial: packeted Task 34 evidence currently covers installer, MIRAI, Flux, and Rudra-family logs; aggregate local/nightly, sanitizer, fuzz, cargo-careful, Kani, Loom, Shuttle, cargo-vet, cargo-geiger, AFL, PG18 sanitizer, and SQLsmith evidence remain gaps until packeted |

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
| TC-019 | `ecaz` AM profiles | `ec_hnsw`, `ec_ivf`, `ec_diskann`, `ec_spire` | Profile metadata selects AM, opclass, embedding type, scan GUC, sweep axis, and reloption set |
| TC-019 | `ecaz` logging | terminal output, `--log-file`, dev SQL `--log-output` | Review evidence can be stored under packet-local artifacts |
| TC-020 benchmark suites | `ecaz bench suite` commands | `run`, `audit`, `status`, `report`, legacy dry-run alias, `--only-tag`, `--resume-from`, `results.jsonl`, thresholds, threshold filters | Configs expand into ordinary `ecaz` commands; manifests support status/report inspection, normalized result rows, threshold assertions, and strict resume safety |
| TC-021 | SPIRE local stores | single store, disabled store, two active stores | Configuration, PID hash placement, diagnostics, and strict/degraded behavior are visible without claiming intra-backend parallelism |
| TC-021 | SPIRE relation options | `storage_format`, `local_store_count`, local store tablespaces, boundary replica count | Valid combinations produce placement diagnostics; invalid bounds reject or surface explicit degraded status |
| TC-022 | SPIRE scan options | `ec_spire.nprobe`, recursive fanout, rerank width, max candidate rows | Effective route budget and candidate limits are visible in diagnostics and bounded in scans |
| TC-023 | SPIRE consistency mode | local strict default, explicit degraded, remote strict | Strict fails closed; degraded reports skipped placements and remote failure metadata |
| TC-024 | SPIRE remote transport | TLS required/disabled for dev, timeout, cancellation, tuple payload shape, version mismatch, remote fanout and payload caps | Remote executor validates endpoint identity, wire version, payload arity/types, cancellation, capacity limits, and fail-closed behavior |
| TC-034 | Hardening analysis lanes | Packeted evidence currently includes installer, MIRAI, Flux, and Rudra-family logs; documented but unpacketed lanes include `hardening-local`, `hardening-nightly-local`, cargo-audit/deny/vet, unsafe audit, Miri, careful, fuzz, Kani, Loom, Shuttle, sanitizers, SQLsmith, cargo-geiger, and AFL | Each lane must record command, gate level, prerequisites, artifact, interpretation rule, and model boundary before it is promoted from gap to completed evidence |

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
| SPIRE local reloptions | Min/max and outside range | `local_store_count`, boundary replica count, local store tablespaces | TC-021 | Boundary pass, outside ERROR or explicit degraded diagnostic |
| SPIRE scan GUCs/reloptions | Min/max and outside range | `nprobe`, recursive fanout, rerank width, max candidate rows | TC-022, TC-023 | Effective values report source; outside range ERROR |
| SPIRE remote limits | Min/max and outside range | remote node fanout, selected PID cap, payload byte cap, timeout, cancellation | TC-024, TC-034 | Strict fail-closed or degraded skip with stable status |
| Hardening optional tools | Missing/present tool states | installed, missing, unsupported platform | TC-034 | Missing tools produce setup text; unsupported platform skips explicitly |

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
| EC-008 | Long benchmark sequence loses provenance across manual shell commands | FR-038 benchmark suites, NFR-007, NFR-009 | TC-020 benchmark suites | Operators cannot audit what ran or identify missing artifacts |
| EC-009 | SPIRE stored heap TID goes stale after UPDATE/HOT movement | FR-048, FR-050, FR-053, FR-059 | TC-020 SPIRE, TC-022, TC-025 | Wrong tuple returned or candidate silently lost |
| EC-010 | SPIRE epoch publish fails after some partition objects are durable | FR-052, FR-054, FR-059, FR-060 | TC-023, TC-025 | Active epoch may point at incompatible object versions |
| EC-011 | Cloud profile left running unattended (forgotten EC2/EBS spend) | NFR-010 | TC-031 | Material AWS spend accumulates silently |
| EC-012 | Loader EC2 worker dies mid-shard during 100M load | FR-047 | TC-029 | Partial corpus loaded; resume must not duplicate rows |
| EC-013 | BIGANN `.fbin` adapter mis-encodes parquet (dim or distance) | FR-046 | TC-028 | Bench results compared against the wrong ground truth |
| EC-014 | Hardening analyzer passes because an optional tool is missing | NFR-004 | TC-034 | False confidence in safety baseline |
| EC-015 | Pure Rust proof is misapplied to pgrx/SPI/libpq callback behavior | NFR-004, FR-058 | TC-024, TC-034 | Incorrect production-readiness claim |
| EC-016 | SPIRE Stage E matrix only checks status strings, not executor behavior | FR-057, FR-058 | TC-024, TC-034 | Distributed failure mode appears covered but fails live |

## Integration Test Matrix

Ecaz has one required local service integration: PostgreSQL itself.

| Integration ID | Purpose | Service | Type | Test Cases | Status |
| --- | --- | --- | --- | --- | --- |
| INT-001 | Extension lifecycle and catalog registration | PostgreSQL 18 | database | TC-003, TC-005, TC-014 | Partial: not run in this docs checkpoint |
| INT-002 | PG17 fallback build/test lane | PostgreSQL 17 | database | TC-014 | Partial: run on demand |
| INT-003 | Real-corpus benchmark surfaces | PostgreSQL plus local corpus files | database/filesystem | TC-015, TC-016, TC-017 | Partial: local evidence exists, product gates deferred |
| INT-004 | CLI operator benchmark and stress workflows | PostgreSQL plus local corpus files | database/filesystem | TC-019 | Partial: docs/spec trace complete; execution run on demand |
| INT-005 | SPIRE local and remote partition-store lifecycle | PostgreSQL 18 plus optional remote PostgreSQL nodes | database | TC-020..TC-025 | Partial: local and PostgreSQL-node v1 behavior is specified and implementation-backed; AWS/RDS evidence deferred |
| INT-006 | Cloud harness end-to-end (provision, install, load, bench, teardown) | AWS (EC2 Graviton, EBS, S3, SSM) plus PostgreSQL 18 | cloud-infrastructure | TC-026..TC-032 | Planned: implementation in progress on `feat/cloud-test-harness` |

## Coverage Gaps

| Gap ID | Description | Risk Level | Mitigation |
| --- | --- | --- | --- |
| GAP-001 | Product benchmark claims for IVF/DiskANN on controlled AWS/RDS-class hardware | Medium | Keep docs labeled local; open dedicated measurement packet before product claims |
| GAP-002 | ReadStream cold-cache speedup verification | Medium | Run PG18 cold-cache matrix when hardware setup is ready |
| GAP-003 | Custom pgstat reset support | Low | Track upstream/local PG18 support for custom-kind reset |
| GAP-004 | HNSW insert throughput decontention | Medium | Track as future Task 13 work |
| GAP-005 | Full requirement-to-individual-test function inventory | Low | Generate from source/test names if a stricter audit packet is needed |
| GAP-006 | Automated CLI README-vs-Clap tree drift check | Low | Add a generated help snapshot or parser-backed docs check if the CLI surface starts changing frequently |
| GAP-007 | Dedicated normalized numeric columns for each metric family | Low | Add typed result fields if downstream plotting needs them beyond string-valued `values` |
| GAP-008 | Source dataset for 10M+ comparable benchmarks not yet ingested | Medium | Resolve in FR-046 dataset-registry adapters (Cohere Wikipedia, LAION subsets, BIGANN) before promoting `10m`/`100m` profiles |
| GAP-009 | Graviton (aarch64) BLAS backend selection for `bench recall` ground-truth matmul | Medium | Verify on first `dev` cloud run; pin in AMI bake if default backend underperforms |
| GAP-010 | SPIRE distributed product-scale evidence | Medium | Keep SPIRE distributed claims scoped to PostgreSQL-node readiness until controlled multi-node/AWS packets land |
| GAP-011 | SPIRE external format freeze tests | Medium | Add binary round-trip and fixture compatibility tests before promising long-term on-disk object compatibility |
| GAP-012 | SPIRE deferred shard SQL and background recovery | Medium | Track cross-shard non-vector query planning, automatic DDL propagation, embedding move updates, and background prepared-xact recovery as explicit follow-on work |
| GAP-013 | SPIRE AC-level test mapping | High | Split TC-020..TC-025 into per-FR or per-AC trace rows before any SPIRE requirement is marked complete |
| GAP-014 | CustomScan lifecycle live coverage | High | Add direct PG18 coverage for Begin/End/ReScan, end-after-cancel cleanup, and MarkPos/RestrPos exclusion |
| GAP-015 | Stage E executor-live fault coverage | High | Convert matrix/string contract rows into live executor tests for each strict/degraded fault category |
| GAP-016 | Task 34 live PG18 hardening lanes | Medium | Packet PG18 sanitizer and SQLsmith runs once a stable PG18 cluster lane is available |
| GAP-017 | Analysis lane promotion criteria | Medium | Define explicit burn-in thresholds before moving report-only Task 34 lanes into PR or nightly gates |
| GAP-018 | Strict per-AC evidence inventory for grouped summary rows | Medium | Split completed summary rows into individual AC-to-TC rows before claiming standards-complete ISO/IEC/IEEE 29148 traceability |
| GAP-019 | Task 34 unpacketed local hardening lane logs | Medium | Packet raw logs for aggregate local/nightly, sanitizer, fuzz, cargo-careful, Kani, Loom, Shuttle, cargo-vet, cargo-geiger, and AFL lanes before marking `TC-034` complete |

## Test Execution Summary

This checkpoint is a docs/spec cleanup. Tests were not run by default under the repository checkpoint policy.

| Category | Total Groups | Implemented / Evidenced | Partial | Gap |
| --- | ---: | ---: | ---: | ---: |
| Unit / pg_test behavior groups | 21 | 15 | 6 | 0 |
| Benchmark / measurement groups | 4 | 1 | 1 | 2 |
| Hardening / analysis groups | 1 | 1 packeted subset | 1 documented-but-unpacketed local lane set plus 1 live PG18/manual lane set | 1 |
| Integration groups | 6 | 0 | 6 | 0 |
