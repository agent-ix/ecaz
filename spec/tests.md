---
id: TM-001
type: TestMatrix
name: ecaz
status: PARTIAL
title: "Test Matrix"
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

The matrix is a correctness baseline, not only a generated inventory. A
requirement row is not standards-complete until every cited acceptance criterion
has a concrete TC/evidence row or an explicit per-AC gap. Grouped rows below are
implementation-backed summaries unless they say per-AC evidence is complete.

| Source | Baseline Evidence | Current Interpretation |
| --- | --- | --- |
| Rust unit and pg_test inventory | SPIRE has broad pure-Rust and PG18 fixture coverage across assignment, metadata, storage, options, scan, coordinator, CustomScan, DML, update, vacuum, and diagnostics modules. | Strong implementation baseline; still requires AC-level trace rows before any standards-complete claim. |
| `reviews/task-30/945-31070-spire-phase12c-coverage-audit` | Independent audit inventoried SPIRE tests, read representative tests for assertion quality, and identified weak or indirect areas such as CustomScan lifecycle, Stage E live coverage, matrix string-only tests, DML row-state assertions, and operator surface coverage. | Treat SPIRE distributed and CustomScan coverage as `Partial` until the named weak areas are closed or explicitly accepted as gaps. |
| `plan/tasks/34-comprehensive-hardening.md`, `docs/hardening.md`, and `reviews/task-34/001-30034-task34-comprehensive-hardening` | Task 34 documented hardening lanes for supply chain, unsafe/static hygiene, Miri, cargo-careful, fuzzing, Kani, Flux, Loom, Shuttle, sanitizers, SQLsmith, Rudra, MIRAI, and aggregate local/nightly targets. Packet-local raw logs in `reviews/task-34/001-30034-task34-comprehensive-hardening` currently support only the installer, MIRAI, Flux, and Rudra-family evidence that appears in that packet manifest. | Adds `TC-034` as the hardening evidence lane for `NFR-004`, but unpacketed local aggregate, sanitizer, fuzz, cargo-careful, Kani, Loom, Shuttle, cargo-vet, cargo-geiger, and AFL claims remain explicit evidence gaps until logs are packeted. Live PG18 sanitizer and SQLsmith lanes remain manually gated gaps. |
| Benchmark reporting standard | `NFR-015` defines identity fields and metric families across AMs, quantizers, storage formats, option sets, and product evidence classes. | The standard itself is implemented, but benchmark rows are only complete after each packet conforms row-by-row. |
| Legacy relationship frontmatter | Active HNSW/core FRs `FR-001..FR-027`, early NFRs, and several stakeholder requirements remain readable through `traces:` or prose fields. | These artifacts remain outside the fully migrated structured relationship graph until `GAP-020` is closed. |

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
| StR-005 | US-012..US-014, FR-028..FR-036 | TC-002, TC-003, TC-004, TC-007..TC-012 | Partial: local implementation surface is backed by grouped evidence; strict per-AC evidence inventory remains `GAP-018` and product scale evidence is deferred |
| StR-005 SPIRE extension | US-018..US-020, US-022, FR-048..FR-060, NFR-013, NFR-014 | TC-020 SPIRE, TC-021..TC-025, TC-034 | Partial: implementation baseline is broad, but AC-level mapping, CustomScan lifecycle proof, Stage E live coverage, and product-scale AWS evidence remain gaps |
| StR-006 | US-015, US-016, US-017 benchmark suites, FR-037, FR-038 benchmark suites, NFR-007..NFR-009, NFR-015 | TC-015, TC-016, TC-019, TC-020 benchmark suites, TC-033 | Partial: product hardware gates are explicit gaps |
| StR-007 cloud | US-021, FR-044..FR-047, NFR-010, NFR-011 | TC-026..TC-032 | Planned: cloud harness implementation begins on `feat/cloud-test-harness` |
| StR-008 distann | FR-075..FR-083, NFR-017..NFR-020 | TC-037..TC-044 | Planned: ec_distann program (Task 161 specs); gate evidence lands with milestones M0..M5 |

### User Story Coverage

| User Story | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| US-001 | US-001-AC-1..4 | TC-001 | Partial: artifact/debug behavior is grouped; strict per-AC evidence inventory remains `GAP-018` |
| US-002 | US-002-AC-1..4 | TC-004, TC-007, TC-010 | Partial: local AM smoke/behavior evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| US-003 | US-003-AC-1..4 | TC-004, TC-006 | Partial: larger parallel build speedups deferred; strict per-AC evidence inventory remains `GAP-018` |
| US-004 | US-004-AC-1..4 | TC-003, TC-014 | Partial: catalog/build behavior evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| US-005 | US-005-AC-1..3 | TC-004 | Partial: product recall-after-vacuum measurements deferred; strict per-AC evidence inventory remains `GAP-018` |
| US-006 | US-006-AC-1..5 | TC-005, TC-017 | Partial: live surface exists; cold-cache speedup measurement deferred |
| US-007 | US-007-AC-1..4 | TC-005 | Partial: local callback/cost evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| US-008 | US-008-AC-1..4 | TC-006, TC-016 | Partial: local implementation landed; AWS/RDS scale evidence deferred |
| US-009 | US-009-AC-1..4 | TC-005, TC-008 | Partial: HNSW/IVF local diagnostics evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| US-010 | US-010-AC-1..4 | TC-004, TC-009, TC-012 | Partial: local AM behavior evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| US-011 | US-011-AC-1..4 | TC-005 | Partial: reset for custom kind remains blocked upstream/local PG18 tree |
| US-012 | US-012-AC-1..3 | TC-002, TC-003, TC-004, TC-007, TC-010 | Partial: current SQL surface evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| US-013 | US-013-AC-1..3 | TC-007, TC-008, TC-009, TC-015 | Partial: local IVF v1 evidence is grouped; strict per-AC evidence inventory remains `GAP-018` and product claims are deferred |
| US-014 | US-014-AC-1..3 | TC-010, TC-011, TC-012, TC-015 | Partial: local DiskANN v1 evidence is grouped; strict per-AC evidence inventory remains `GAP-018` and product claims are deferred |
| US-015 | US-015-AC-1..4 | TC-015, TC-016, TC-033 | Partial: product benchmark claim lane is a planned gate |
| US-016 | US-016-AC-1..3 | TC-019 | Partial: docs/spec traceability is grouped; command execution tests run on demand and strict per-AC evidence inventory remains `GAP-018` |
| US-017 benchmark suites | US-017-AC-1..5 | TC-020 benchmark suites, TC-033 | Partial: first auto-runner surface is implemented, but strict per-AC evidence inventory remains `GAP-018` and richer thresholds are deferred |
| US-018 | US-018-AC-1..6 | TC-021, TC-022, TC-023 | Partial: relation-backed local stores, PID hash placement, store diagnostics, strict/degraded handling, and sequential backend read scheduling are implemented, but strict per-AC evidence inventory remains `GAP-018` and true parallel local-store execution is deferred |
| US-019 | US-019-AC-1..6 | TC-023, TC-024 | Partial: CustomScan distributed reads, placement-aware dispatch, typed remote tuple payloads, and origin-node visibility are implemented, but strict per-AC evidence inventory remains `GAP-018` and AWS product evidence is deferred |
| US-020 | US-020-AC-1..6 | TC-023, TC-025 | Partial: epoch publication, maintenance hooks, diagnostics, and coordinator DML/2PC recovery are implemented, but strict per-AC evidence inventory remains `GAP-018` and background prepared-xact recovery is deferred |
| US-022 | US-022-AC-1..6 | TC-020 SPIRE, TC-021, TC-022, TC-025 | Partial: local build/publish/search lifecycle and operator-visible maintenance are implemented, but strict per-AC evidence inventory remains `GAP-018` and long-running scale evidence is deferred |
| US-021 | US-021-AC-1..5 | TC-026, TC-029, TC-030 | Planned: implementation in progress on cloud branch |

### Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
| --- | --- | --- | --- |
| FR-001..FR-006 | Type, I/O, encode, scoring, operators | TC-001, TC-002, TC-003 | Partial: current unit/pg_test coverage is grouped; strict per-AC evidence inventory remains `GAP-018` and legacy frontmatter migration remains `GAP-020` |
| FR-007..FR-013, FR-016..FR-018 | HNSW layout/build/scan/vacuum/WAL/insert/scoring | TC-001, TC-004, TC-013 | Partial: old HNSW product benchmark rows need refreshed evidence |
| FR-014 | FR-014-AC-1..5 block-kernel scoring, ISA dispatch, width buckets, counters, and correctness anchors | TC-035, TC-033 | Partial: architecture and reporting standard are specified; Task 99 completeness matrix, Task 102 ARM (Graviton 4 NEON/SVE2) evidence, and the Task 103 Intel lane (int8_approx32 AVX2, rabitq32 validation, recorded tiled_lut/hamming dispositions) remain explicit gates |
| FR-015 | FR-015-AC-1..10 ProdQuantizer math plus index-local QuantCodec adapter boundary | TC-001, TC-035 | Partial: deterministic encode/score surface is implementation-backed, but AC-10 requires cross-AM adapter audit evidence before standards-complete closure |
| FR-019 | ReadStream integration | TC-005, TC-017 | Partial: behavior coverage exists; speedup evidence deferred |
| FR-020 | Planner cost estimation | TC-005 | Partial: local modeled/live cost evidence is grouped; strict per-AC evidence inventory remains `GAP-018` and legacy frontmatter migration remains `GAP-020` |
| FR-021 | Parallel index build | TC-006, TC-016 | Partial: scale measurement deferred |
| FR-022 | Vacuum implementation | TC-004, TC-009, TC-012 | Partial: local behavior evidence is grouped; strict per-AC evidence inventory remains `GAP-018` and legacy frontmatter migration remains `GAP-020` |
| FR-023 | Strategy translation callbacks | TC-005, TC-008 | Partial: PG18 callback evidence is grouped; strict per-AC evidence inventory remains `GAP-018` and legacy frontmatter migration remains `GAP-020` |
| FR-024 | Custom EXPLAIN | TC-005, TC-008 | Partial: HNSW/IVF diagnostics evidence is grouped; strict per-AC evidence inventory remains `GAP-018` and legacy frontmatter migration remains `GAP-020` |
| FR-025 | Custom statistics | TC-005 | Partial: shared reset path remains a known blocker |
| FR-026 | PG18 module identity | TC-005, TC-014 | Partial: PG18 build evidence is grouped; strict per-AC evidence inventory remains `GAP-018` and legacy frontmatter migration remains `GAP-020` |
| FR-027 | pgrx PG18 support | TC-014 | Partial: current build configuration evidence is grouped; strict per-AC evidence inventory remains `GAP-018` and legacy frontmatter migration remains `GAP-020` |
| FR-028 | FR-028-AC-1..4 | TC-002, TC-003 | Partial: canonical `ecvector` evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| FR-029 | FR-029-AC-1..4 | TC-003 | Partial: SQL bootstrap evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| FR-030 | FR-030-AC-1..4 | TC-004, TC-005, TC-006 | Partial: large-build measurement deferred |
| FR-031 | FR-031-AC-1..3 | TC-007 | Partial: local IVF build/storage evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| FR-032 | FR-032-AC-1..3 | TC-008 | Partial: local IVF scan/rerank/cost evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| FR-033 | FR-033-AC-1..3 | TC-009 | Partial: local IVF insert/vacuum/admin evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| FR-034 | FR-034-AC-1..3 | TC-010 | Partial: local DiskANN build/storage evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| FR-035 | FR-035-AC-1..3 | TC-011 | Partial: local DiskANN scan/prefilter/rerank evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| FR-036 | FR-036-AC-1..3 | TC-012 | Partial: local DiskANN insert/vacuum/diagnostics evidence is grouped; strict per-AC evidence inventory remains `GAP-018` |
| FR-037 | FR-037-AC-1..4 | TC-019 | Partial: docs/spec traceability is grouped; CLI unit execution was not run in this docs checkpoint and strict per-AC evidence inventory remains `GAP-018` |
| FR-038 benchmark suites | FR-038-AC-1..10 | TC-020 benchmark suites, TC-033, TC-036 | Partial: first auto-runner surface is implemented, but strict per-AC evidence inventory, backend-profile preflight proof, and full schema-driven report generation remain iterative |
| FR-039..FR-043 tombstones | No active ACs | Spec inspection | Superseded: retained tombstone files preserve immutable ID history and point to `FR-048..FR-060` replacements |
| FR-048 | FR-048-AC-1..8 | TC-020 SPIRE, TC-021, TC-024, TC-025 | Partial: domain model, identities, epochs, placement, and read/write boundary definitions are specified, but strict per-AC evidence inventory remains `GAP-018` |
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
| FR-060 | FR-060-AC-1..9 | TC-021, TC-023, TC-024, TC-025 | Partial: diagnostics and fail-closed reporting exist; operator surface drift checks and matrix-to-live coverage remain gaps |
| FR-061 IVF persisted format | FR-061-AC-1..3 | TC-007 | Partial: spec-only schema transcription from `src/am/ec_ivf/page.rs`; independent-decode and rejection-path evidence remain `GAP-018`-style inventory work |
| FR-062 DiskANN persisted format | FR-062-AC-1..3 | TC-010, TC-012 | Partial: spec-only schema transcription from `src/am/ec_diskann/{page,tuple}.rs`; rejection-path evidence inventory pending |
| FR-063 block-kernel counter snapshot | FR-063-AC-1..3 | TC-035 | Partial: field names and label sets pinned; round-trip parity test between snapshot, log line, and results row not yet packeted |
| FR-064 suite config schema | FR-064-AC-1..3 | TC-020 benchmark suites, TC-036 | Partial: schema documented from `SuiteConfig`; checked-in-suite parse audit not yet packeted |
| FR-065 suite manifest schema | FR-065-AC-1..3 | TC-020 benchmark suites, TC-036 | Partial: schema documented from `SuiteManifest`; backend-preflight proof shares TC-036 gaps |
| FR-066 suite results row schema | FR-066-AC-1..3 | TC-020 benchmark suites, TC-033, TC-035 | Partial: row shape documented from `ResultRow`; FR-063 field-parity test not yet packeted |
| FR-067 DiskANN scan pipeline | FR-067-AC-1..3 | TC-011, TC-035 | Partial: stage decomposition specified; batch-on/off A/B and stage-attribution packet evidence pending |
| FR-068 IVF scan pipeline | FR-068-AC-1..3 | TC-008, TC-035 | Partial: stage decomposition specified; pruning-vs-batch axis evidence pending |
| FR-069 IVF parallel build | FR-069-AC-1..3 | TC-007, TC-016 | Partial: protocol specified; serial/parallel equivalence evidence lives with FR-031-AC-4 fixtures |
| FR-070 suite run lifecycle | FR-070-AC-1..3 | TC-020 benchmark suites, TC-036 | Partial: ordering guarantees specified; preflight/resume CLI evidence shares TC-036 gaps |
| FR-071 HNSW configuration | FR-071-AC-1..2 | TC-004, TC-035 | Partial: GUC/reloption inventory matches `register_gucs()` at authoring time; drift audit not automated |
| FR-072 IVF configuration | FR-072-AC-1..2 | TC-008 | Partial: inventory matches `register_gucs()` at authoring time, including Task 51 adaptive/SoA switches; drift audit not automated |
| FR-073 DiskANN configuration | FR-073-AC-1..2 | TC-011 | Partial: inventory matches `register_gucs()` at authoring time, including candidate_batch_scoring and scan_profile_notice; drift audit not automated |
| FR-074 QuantCodec scoring contract | FR-074-AC-1..3 | TC-035 | Partial: trait surface pinned from `src/am/common/quant_codec.rs`; cross-AM adapter audit shares the FR-015-AC-10 gap |
| FR-044 | FR-044-AC-1..4 | TC-026, TC-030 | Planned: cloud command surface and idempotence |
| FR-045 | FR-045-AC-1..4 | TC-027 | Planned: terraform module and profile selection |
| FR-046 | FR-046-AC-1..3 | TC-028 | Planned: dataset registry and parquet staging |
| FR-047 | FR-047-AC-1..4 | TC-029, TC-032 | Planned: in-VPC parallel corpus load |
| FR-075 | FR-075-AC-1..4 | TC-037 | Planned: ec_distann AM surface (`src/tests/ec_distann_basic.rs`) |
| FR-076 | FR-076-AC-1..4, FR-076-CON-1..2 | TC-037, TC-044 | Planned: record round-trip + identity stability in TC-037; CON-1 storage budget measured in TC-044 |
| FR-077 | FR-077-AC-1..4, FR-077-CON-1..4 | TC-038, TC-039 | Planned: proptest invariants (TC-038) + stitched-vs-monolithic recall A/B (TC-039) |
| FR-078 | FR-078-AC-1..3 | TC-040 | Planned: placement determinism + roster-change epoch behavior |
| FR-079 | FR-079-AC-1..4 | TC-040 | Planned: expand_nodes contract incl. epoch-fingerprint rejection and placement-error disambiguation |
| FR-080 | FR-080-AC-1..4, FR-080-CON-1 | TC-037, TC-041 | Planned: determinism/reachability in TC-037; C recall-sensitivity measurement in TC-041 |
| FR-081 | FR-081-AC-1..5 | TC-041 | Planned: 2-node result identity, BW×H cap assertion, dedupe, early-exit equivalence, EXPLAIN counters |
| FR-082 | FR-082-AC-1..3 | TC-042 | Planned: lifecycle drills (epoch-swap-under-load, fingerprint retry, retirement gating) |
| FR-083 | FR-083-AC-1..6 | TC-043 | Planned: tombstone/vacuum, interim posture, incremental insert parity + mid-insert fault drills |

### Non-Functional Requirement Coverage

| NFR | Verification Method | Evidence/Test Cases | Status |
| --- | --- | --- | --- |
| NFR-001 | SQL latency benchmarks | TC-015, TC-016 | Partial: local rows exist; product latency claims need controlled hardware |
| NFR-002 | Storage-size measurement | TC-015 | Partial: local HNSW/IVF/DiskANN rows exist; full product accounting deferred |
| NFR-003 | Recall measurement | TC-015 | Partial: local recall rows exist; product claim gate deferred |
| NFR-004 | NFR-004-AC-1..5 | TC-013, TC-034 | Partial: Task 34 documents the lane surface, but only packet-local raw logs count as completed evidence; unpacketed local lanes, PG18 sanitizer, and SQLsmith remain gaps |
| NFR-005 | Build and CI | TC-014 | Partial: static docs checkpoint did not run build/test commands |
| NFR-006 | Async I/O cold-cache performance | TC-017 | Gap: measurement deferred |
| NFR-007 | Benchmark provenance | TC-015, TC-020 benchmark suites, TC-036 | Partial: review-packet citations exist, but backend-profile proof and legacy benchmark summary rows remain outside NFR-015 row-level conformance until `GAP-021` closes |
| NFR-008 | Scale boundary | TC-016 | Partial: policy is specified; execution remains deferred and strict per-AC evidence inventory remains `GAP-018` |
| NFR-009 | CLI drift and artifact discipline | TC-019 | Partial: docs/spec traceability exists; command-tree execution audit is deferred to CLI tests and strict per-AC evidence inventory remains `GAP-018` |
| NFR-010 | Cloud cost discipline (status reporting, no NAT, --confirm-cost gate) | TC-031 | Planned: cloud harness implementation in progress |
| NFR-011 | Cloud corpus load throughput targets | TC-032 | Planned: targets baseline once first `1m` run lands |
| NFR-012 | Cloud throughput targets | TC-016, TC-032 | Partial: targets are specified; product evidence is gated on controlled cloud runs |
| NFR-013 | SPIRE local readiness and capacity | TC-020 SPIRE, TC-021, TC-022, TC-023, TC-025 | Partial: implementation traceability exists; full capacity envelope needs controlled local storage evidence |
| NFR-014 | SPIRE transport security and operations | TC-024, TC-025 | Partial: v1 contract specifies TLS, timeout, cancellation, and observability behavior; deployment evidence deferred |
| NFR-015 | Benchmark reporting standard | TC-033, TC-035, TC-036 | Partial: standard is specified; existing and future benchmark rows must conform packet-by-packet, including block-kernel counters and backend provenance, before being marked complete |
| NFR-017 | distinct_recall/latency gate vs release IVF anchor, 10k/50k/100k, 3-worker | TC-044 | Planned: pre-registered `ecaz bench suite` matrix on the Task 146 anchor protocol; release-guarded distann-pipeline step kind |
| NFR-018 | Space amplification ≤4× raw vector bytes | TC-044 | Planned: storage step ratio row per scale, plus transient build peak in epoch manifest |
| NFR-019 | Per-query expansion ≤ BW×H, corpus-independent | TC-041, TC-044 | Planned: per-cell counter assertion + cross-scale expanded-count ratio row |
| NFR-020 | Fault behavior: correct-or-error, never silent partials | TC-042, TC-043 | Planned: full drill matrix (reused + hop_round_failure_mid_beam, missing_node_record, placement_drift, mid-insert) |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
| --- | --- | --- | --- | --- | --- |
| TC-001 | `tqvector` artifact layout, I/O, encode, scoring | Unit / pg_test | P0 | FR-001..FR-006, FR-013..FR-018 | Implemented |
| TC-002 | `ecvector` typmod, I/O, casts, encode defaults | Unit / pg_test | P0 | US-012, FR-028 | Implemented |
| TC-003 | SQL bootstrap registers extension objects | pg_test / catalog inspection | P0 | US-004, FR-012, FR-029 | Implemented |
| TC-004 | HNSW build, scan, insert, vacuum, storage formats | pg_test | P0 | US-002, US-003, US-005, FR-007..FR-018, FR-030, FR-071 | Implemented |
| TC-005 | PG18 planner, EXPLAIN, stats, module identity | pg_test / inspection | P0 | US-006, US-007, US-009, US-011, FR-019..FR-027, FR-030 | Partial: custom stats reset blocked |
| TC-006 | HNSW parallel build and DSM graph assembly | pg_test / benchmark | P1 | US-008, FR-021, FR-030 | Partial: local implementation landed, scale evidence deferred |
| TC-007 | IVF build, reloptions, metadata, storage formats | pg_test | P0 | US-013, FR-031, FR-061, FR-069 | Implemented |
| TC-008 | IVF scan, GUC overrides, rerank, cost, EXPLAIN | pg_test | P0 | US-013, FR-032, FR-068, FR-072 | Implemented |
| TC-009 | IVF insert, vacuum, admin/drift snapshots | pg_test | P0 | US-013, FR-033 | Implemented |
| TC-010 | DiskANN build, unit-normalized contract, graph storage | pg_test | P0 | US-014, FR-034, FR-062 | Implemented |
| TC-011 | DiskANN scan, prefilter, list-size override, rerank | pg_test | P0 | US-014, FR-035, FR-067, FR-073 | Implemented |
| TC-012 | DiskANN insert, vacuum repair, diagnostics | pg_test | P0 | US-014, FR-036, FR-062 | Implemented |
| TC-013 | Safety, WAL discipline, unsafe/fuzz/license review | Unit / fuzz / inspection | P1 | NFR-004, FR-011 | Partial: run explicitly when risk warrants |
| TC-014 | PG18 primary and PG17 compatibility builds | CI / build | P0 | US-004, FR-026, FR-027, NFR-005 | Partial: not run in this docs checkpoint |
| TC-015 | Local benchmark provenance for HNSW/IVF/DiskANN | Review packet / docs audit | P1 | US-015, NFR-001, NFR-002, NFR-003, NFR-007 | Implemented for current docs |
| TC-016 | AWS/RDS-class product benchmark gate | Benchmark | P2 | NFR-008, US-015 | Gap: deferred |
| TC-017 | ReadStream cold-cache speedup gate | Benchmark | P2 | NFR-006, FR-019 | Gap: deferred |
| TC-018 | HNSW insert decontention follow-up | Benchmark / implementation | P2 | Future Task 13 | Gap: future work |
| TC-019 | `ecaz` CLI command tree, profiles, logging, and docs links | Unit / docs audit | P1 | US-016, FR-037, NFR-009 | Implemented for docs traceability; CLI tests run on demand |
| TC-020 benchmark suites | `ecaz bench suite` dry-run, execution manifest, audit, status, report, and results extraction | Unit / CLI smoke | P1 | US-017 benchmark suites, FR-038 benchmark suites, FR-064, FR-065, FR-066, FR-070, NFR-007, NFR-009 | Implemented for first auto-runner surface; backend profile preflight remains under TC-036 |
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
| TC-033 | Benchmark reporting standard docs/spec audit | Docs / spec audit | P1 | US-015, US-017, FR-038, NFR-015 | Implemented for the standard; future benchmark packets apply row-level block-kernel and backend-provenance fields |
| TC-034 | Task 34 hardening and analysis lanes | Static analysis / fuzz / model checking / sanitizer / supply-chain audit | P0 | NFR-004, FR-011, FR-049, FR-050, FR-051, FR-052, FR-053, FR-054, FR-055, FR-056, FR-057, FR-058, FR-059 | Partial: packeted Task 34 evidence currently covers installer, MIRAI, Flux, and Rudra-family logs; aggregate local/nightly, sanitizer, fuzz, cargo-careful, Kani, Loom, Shuttle, cargo-vet, cargo-geiger, AFL, PG18 sanitizer, and SQLsmith evidence remain gaps until packeted |
| TC-035 | QuantCodec block-kernel completeness matrix | Unit / benchmark packet / docs audit | P0 | FR-014, FR-015, FR-030, FR-032, FR-035, FR-063, FR-066, FR-067, FR-068, FR-071, FR-074, NFR-015 | Partial: target matrix is specified; Task 99, Task 102 ARM evidence, the Task 103 Intel lane, Graviton 4 vector-length, and deferred hardware cells remain gaps until packeted |
| TC-036 | Benchmark suite backend-profile preflight | CLI unit / suite audit | P0 | FR-038, FR-064, FR-065, FR-070, NFR-007, NFR-015 | Partial: suite manifests must prove release/debug backend selection for latency and recall rows before product benchmark claims |
| TC-037 | ec_distann single-node AM surface, record format, head index; M0 bench evidence (ec_diskann parity A/B, head-cap C sensitivity, D7 codec comparison, D1 storage ratio) | Unit / pg_test (`src/tests/ec_distann_basic.rs`) + `ecaz bench suite` (M0 cells) | P0 | FR-075, FR-076, FR-080, NFR-018 | Planned (M0) |
| TC-038 | Stitch correctness property suite (degree ≤ R, vec_id uniqueness, medoid reachability, idempotence, α-prune invariant) | proptest (`src/am/ec_distann/`) | P0 | FR-077-CON-1..3, FR-077-AC-2, FR-077-AC-4 | Planned (M1) |
| TC-039 | Stitched-vs-monolithic build recall A/B at 100k | Benchmark (`ecaz bench suite`) | P0 | FR-077-AC-1, FR-077-AC-3 | Planned (M1) |
| TC-040 | Hash placement determinism + remote expansion protocol contract | pg_test (`src/tests/ec_distann_remote.rs`) | P0 | FR-078, FR-079 | Planned (M2) |
| TC-041 | Hop-round orchestration: 2-node result identity, BW×H cap, dedupe, early-exit equivalence, EXPLAIN counters, head-index C sensitivity | pg_test + 2-node fixture + bench counters | P0 | FR-080, FR-081, NFR-019 | Planned (M2) |
| TC-042 | Epoch lifecycle + multinode fault drill matrix (reused cases + hop_round_failure_mid_beam, missing_node_record, placement_drift) | Multinode fixture drills | P0 | FR-082, NFR-020 | Planned (M3) |
| TC-043 | DML path: tombstone/vacuum, interim insert posture, incremental distributed insert parity, mid-insert fault + concurrency drills | pg_test + multinode drills | P1 | FR-083, NFR-020 | Planned (M3/M5) |
| TC-044 | distann bench gate: 10k/50k/100k four-way comparison (ec_distann / IVF / HNSW / best-SPIRE) on the Task 146 protocol, release-guarded distann-pipeline step, matched-recall rule per NFR-017, informational netem H×RTT run, multinode storage summation, min-BW×H-for-recall row; M5 insert-parity bench cell (FR-083-AC-4) | Benchmark (`ecaz bench suite`) | P0 | NFR-017, NFR-018, NFR-019, FR-076-CON-1, FR-083-AC-4, StR-008 | Planned (M4 gate; prerequisite: task-138 distinct_recall + task-146 anchors merged) |

## Option Permutation Matrix

| Test Case | Option Set | Required Coverage | Expected Behavior |
| --- | --- | --- | --- |
| TC-037 | `ec_distann.neighbor_code_format` | `grouped_pq` (default), `rabitq`, `turboquant` | All codecs build/scan correctly; recall/storage per codec recorded at M0 (ADR-085 D7) |
| TC-038 | `ec_distann.closure_epsilon` | 0 (single shard assignment), default, high overlap | Build succeeds; duplication factor recorded; stitch output invariants hold at every value |
| TC-041 | `ec_distann.beam_width` × `ec_distann.hop_rounds` | low/default/high BW × H combinations | Expansion counter ≤ BW×H in every combination; recall monotone non-decreasing in BW and H |
| TC-004 | `ec_hnsw.storage_format` | `turboquant`, `pq_fastscan`, `rabitq` | Valid formats build/scan; incompatible live storage-format changes reject until rebuild |
| TC-004 | `ec_hnsw.ef_search` | relation default, session override, reset | Effective scan breadth follows session override when set |
| TC-004 | `ec_hnsw.turboquant_exact_score_mode` | `exact`, `full_lut`, `tiled_lut`, `int8_approx` | All four modes return correct ordered results; compressed modes emit their own `quant_kind` counter rows (`FR-071`) |
| TC-004 | `ec_hnsw.candidate_batch_scoring` | on, off | Identical result sets; counter attribution moves between kernel and per-candidate routes (`FR-071-AC-2`) |
| TC-006 | `ec_hnsw.enable_parallel_build_concurrent_dsm` | true, false | true uses concurrent DSM path when eligible; false uses diagnostic fallback |
| TC-007 | `ec_ivf.storage_format` | `auto`, `turboquant`, `pq_fastscan`, `rabitq` | Valid formats build; invalid strings reject |
| TC-007 | `ec_ivf.rerank` | `auto`, `off`, `heap_f32`, `source_column` | First three supported; `source_column` rejected in v1 |
| TC-008 | `ec_ivf.nprobe`, `ec_ivf.rerank_width` | relation, session, auto | Effective scan settings report correct source |
| TC-008 | `ec_ivf.adaptive_nprobe` (+ `score_gap_micros`, `score_margin_ratio_bps`) | off, gap signal, ratio signal | Off never reduces nprobe; enabled reduction is deterministic for a given query/frontier and only when the configured signal triggers (`FR-072`) |
| TC-008 | `ec_ivf.scratch_soa_batch_decode` | on, off | Identical result sets; decode batching is a latency-only axis recorded per `FR-072-CON-2` |
| TC-010 | `ec_diskann.storage_format` | `pq_fastscan` | Valid; other values reject |
| TC-011 | `ec_diskann.prefilter_kind` | `auto`, `binary_sidecar`, `grouped_pq` | Selects persisted sidecar or grouped-PQ fallback as requested |
| TC-011 | `ec_diskann.list_size` | relation, session override, reset | Effective scan breadth reports correct source |
| TC-011 | `ec_diskann.candidate_batch_scoring` | on, off | Identical result sets; `surface=diskann` kernel counter rows appear only when on (`FR-073-AC-2`) |
| TC-011 | `ec_diskann.scan_profile_notice` | on, off | On emits one per-query stage-timing NOTICE; results and counters unchanged (`FR-073`) |
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
| TC-035 | Block-kernel dispatch matrix | quant kind, AM surface, ISA label, width bucket, scalar anchor | Matrix rows distinguish exact block32, partial/octet, scalar remainder, absent, and deferred cells |
| TC-036 | Benchmark backend profile states | release backend, debug backend without override, debug backend with explicit override | Latency/recall suites fail fast on debug backend unless explicitly allowed, and manifests preserve backend provenance |

## Constraint Boundary Tests

| Constraint | Boundary Type | Test Value | Test Case | Expected |
| --- | --- | --- | --- | --- |
| distann `graph_degree` (R) out-degree | Max | union of shard edges > R | TC-038 | Stitch re-prunes to exactly ≤ R |
| distann reloption ranges (`graph_degree`, `head_index_cap`, `hop_rounds`, `beam_width`) | Below-min / above-max | invalid values at CREATE INDEX / SET | TC-037 | ERROR with descriptive message |
| distann expansion cap | Max | query configured at BW×H boundary | TC-041 | Counter never exceeds BW×H |
| distann space amplification | Max | 100k storage ratio | TC-044 | ≤ 4.0× raw vector bytes (NFR-018 threshold) |
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
| EC-019 | Hop round k of H fails after k−1 succeeded (partial beam) | FR-081, NFR-020 | TC-042 | Silent recall degradation presented as complete results |
| EC-020 | vec_id hash collision at build time and at incremental insert (ADR-085 D6, FR-083) | FR-076, FR-083 | TC-037, TC-043 | Two logical rows alias one graph node; wrong results |
| EC-021 | All frontier candidates owned by one node (batch skew) | FR-078, FR-081 | TC-041 | Round serialization or per-node batch overflow |
| EC-022 | Query during epoch swap; expansion against retired epoch | FR-082 | TC-042 | Mixed-epoch results or reads of reclaimed storage |
| EC-023 | Tombstoned neighbor mid-traversal; vacuumed neighbor edge | FR-083 | TC-043 | Traversal dead-ends or errors on reclaimed records |
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
| EC-017 | Block-kernel partial widths fall back silently without counter attribution | FR-014, FR-030, FR-032, FR-035, NFR-015 | TC-035 | Scoring-share claims misrepresent production AM batch distributions |
| EC-018 | Latency/recall benchmark uses a debug backend without manifest evidence | FR-038, NFR-007, NFR-015 | TC-036 | Product benchmark claim is inflated or not reproducible |

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
| GAP-020 | Legacy structured relationship frontmatter migration | Medium | Migrate active legacy `FR-001..FR-027`, early NFRs, and stakeholder requirements from `traces:`/prose links to `artifact_type` plus semantic `relationships:` before claiming whole-spec ISO/IEC/IEEE 42010 or IEEE 828 graph completeness |
| GAP-021 | Legacy benchmark row conformance to NFR-015 | Medium | Either convert legacy HNSW/IVF/DiskANN benchmark tables to row-level NFR-015 fields or keep them explicitly labeled as local summary rows outside standards-complete comparisons |
| GAP-022 | Block-kernel Task 99 completeness matrix not yet packeted | High | Packet row-level quant-kind, AM-surface, ISA, width-bucket, scalar-anchor, recall, and latency evidence before accepting ADR-077 |
| GAP-023 | LUT32 SIMD closure partially evidenced | Medium | AVX2 kernels are landed, measured, and packeted (Task 102 packets 001-002); close Task 102 with Graviton 4 NEON/SVE2 hardware evidence before publishing LUT32 completion claims |
| GAP-024 | Graviton 4 vector-length and deferred hardware cells not yet packeted | Medium | Require measured `sve`/`sve2` vector-length labels and explicit absent/deferred cells before ARM production claims |
| GAP-025 | Task 103 Intel AVX2 lane not yet closed | Medium | Land the int8_approx32 AVX2 kernel (AC1), record the tiled_lut32 retire/deprioritize and hamming32 skip dispositions in the matrix (AC2/AC3), and validate/bench rabitq32 on Intel (AC4) before Intel completeness claims |

## Test Execution Summary

This checkpoint is a docs/spec cleanup. Tests were not run by default under the repository checkpoint policy.

| Category | Total Groups | Implemented / Evidenced | Partial | Gap |
| --- | ---: | ---: | ---: | ---: |
| Unit / pg_test behavior groups | 21 | 15 | 6 | 0 |
| Benchmark / measurement groups | 6 | 1 | 3 | 2 |
| Hardening / analysis groups | 1 | 1 packeted subset | 1 documented-but-unpacketed local lane set plus 1 live PG18/manual lane set | 1 |
| Integration groups | 6 | 0 | 6 | 0 |
