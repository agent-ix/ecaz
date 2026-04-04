# Implementation Plan: tqvector

This plan is derived from the current `spec/` set for `tqvector`, with dependency edges inferred primarily from `traces:` frontmatter and validated against the requirement text. The spec does not consistently use `relationships:` arrays for FR files, so this plan uses the explicit trace graph plus normative references in the requirement bodies.

## Requirements Summary

### Stakeholder Requirements

- [ ] **StR-001**: Native compressed vector storage and ANN search inside PostgreSQL.
- [ ] **StR-002**: MIT-licensed extension owned by Agent-IX with permissive dependencies only.
- [ ] **StR-003**: Partition-local HNSW operation with no cross-partition coupling.

### Functional Requirements

- [ ] **FR-001**: Register `tqvector` type and binary datum layout.
- [ ] **FR-002**: Text I/O for `tqvector`.
- [ ] **FR-003**: Binary send/receive protocol for `tqvector`.
- [ ] **FR-004**: `encode_to_tqvector` API from fp32 arrays.
- [ ] **FR-005**: Code-to-code inner product function.
- [ ] **FR-006**: SQL operators and operator class.
- [ ] **FR-007**: HNSW page layout and tuple/page invariants.
- [ ] **FR-008**: HNSW bulk build callbacks.
- [ ] **FR-009**: HNSW scan callbacks and `ef_search` behavior.
- [ ] **FR-010**: HNSW vacuum callbacks.
- [ ] **FR-011**: WAL safety via GenericXLog.
- [ ] **FR-012**: SQL bootstrap and extension packaging.
- [ ] **FR-013**: Two-stage quantization pipeline.
- [ ] **FR-014**: SIMD acceleration with scalar fallback.
- [ ] **FR-015**: `ProdQuantizer` orchestrator and core encode/decode/score APIs.
- [ ] **FR-016**: HNSW online insert callbacks and insert-drift statistics.
- [ ] **FR-017**: Prepared-query inner product function.
- [ ] **FR-018**: Negative score wrapper functions.

### Non-Functional Requirements

- [ ] **NFR-001**: Query latency and throughput targets.
- [ ] **NFR-002**: Storage compression and index-size accounting.
- [ ] **NFR-003**: Recall quality and benchmark methodology.
- [ ] **NFR-004**: Safety and stability.
- [ ] **NFR-005**: Build and CI quality gates.

## Dependency Graph

### Core dependency edges

- `FR-013 -> FR-015`
  Reason: `ProdQuantizer` is the implementation of the quantization/scoring formulas.
- `FR-015 -> FR-004, FR-005, FR-017`
  Reason: the public encode and scoring APIs rely on the orchestrator.
- `FR-005 + FR-017 -> FR-018 -> FR-006`
  Reason: negative wrappers depend on the positive functions; SQL operators depend on wrapper/function registration.
- `FR-001 -> FR-002, FR-003`
  Reason: text/binary I/O depend on the datum format and type registration.
- `FR-007 -> FR-008, FR-009, FR-010, FR-016`
  Reason: build/scan/vacuum/insert all depend on stable page and tuple layout.
- `FR-011 -> FR-008, FR-010, FR-016`
  Reason: all index write paths must use GenericXLog.
- `FR-017 + FR-007 + FR-015 -> FR-009`
  Reason: scan uses prepared-query scoring against page tuples.
- `FR-015 + FR-007 -> FR-016`
  Reason: insert uses code-to-code scoring against page tuples.
- `FR-012`
  Reason: bootstrap/packaging is mostly independent, but final SQL registration depends on available functions and operators.
- `FR-014`
  Reason: can start after scalar quantizer/scoring APIs exist; not a blocker for correctness-first implementation.

### Cross-cutting constraints

- `NFR-004` applies to all parser, page, WAL, and unsafe code paths.
- `NFR-005` applies to every merge point.
- `NFR-001`, `NFR-002`, and `NFR-003` are verification-heavy and should not block initial correctness unless a design choice depends on benchmark evidence.

## Critical Path

The shortest path to an end-to-end usable extension is:

1. `FR-013` quantization math
2. `FR-015` orchestrator and core packing/scoring helpers
3. `FR-001`, `FR-002`, `FR-003` type + I/O
4. `FR-004`, `FR-005`, `FR-017`, `FR-018` SQL-callable encode/score functions
5. `FR-006`, `FR-012` operators and extension packaging
6. `FR-007` page layout
7. `FR-008` bulk build
8. `FR-009` scan
9. `FR-011` WAL guarantees hardened across all write paths
10. `FR-010` vacuum
11. `FR-016` online insert
12. `FR-014` SIMD optimization and full benchmark passes for `NFR-001` to `NFR-003`

## Parallel Execution Plan

### Phase 0: Scaffolding and repo setup

- [ ] Create crate/module skeleton for:
  - type and I/O
  - quantizer core
  - SQL function bindings
  - HNSW AM modules
  - benchmark and test harnesses
- [ ] Set up CI skeleton for `fmt`, `clippy`, unit tests, `cargo pgrx test`, and license checks.

Parallelism:
- Workstream A: test harness and CI scaffolding
- Workstream B: crate/module skeleton

### Phase 1: Scalar quantizer core

- [ ] Implement `FR-013` scalar math:
  - SRHT/FWHT
  - codebook generation
  - MSE packing
  - QJL packing
  - decode approximation helpers
- [ ] Implement `FR-015` scalar `ProdQuantizer`:
  - constructor
  - encode/decode
  - query preparation
  - `score_ip_encoded`
  - `score_ip_encoded_lite`
  - cache hooks

Parallelism:
- Workstream A1: transforms and codebook generation
- Workstream A2: packing/unpacking primitives
- Workstream A3: orchestrator shell and cache structure

Merge point:
- `FR-015` cannot be completed until A1 and A2 land.

### Phase 2: Datum and SQL function surface

- [ ] Implement `FR-001` datum pack/unpack and type registration.
- [ ] Implement `FR-002` text I/O.
- [ ] Implement `FR-003` binary send/receive.
- [ ] Implement `FR-004` `encode_to_tqvector`.
- [ ] Implement `FR-005`, `FR-017`, `FR-018` SQL-visible scoring functions.
- [ ] Implement `FR-006` operators and operator class.
- [ ] Implement `FR-012` extension packaging and SQL install/uninstall.

Parallelism:
- Workstream B1: datum/type/I/O (`FR-001` to `FR-003`)
- Workstream B2: encode and score functions (`FR-004`, `FR-005`, `FR-017`, `FR-018`)
- Workstream B3: SQL DDL/bootstrap (`FR-006`, `FR-012`)

Merge points:
- B2 depends on Phase 1.
- B3 depends on exported SQL functions from B1/B2.

### Phase 3: Page layout and storage engine base

- [ ] Implement `FR-007` page structs, tuple codecs, fit checks, and level-cap logic.
- [ ] Implement `FR-011` GenericXLog wrapper utilities and write discipline.

Parallelism:
- Workstream C1: tuple/page layout and page inspection helpers
- Workstream C2: WAL/write abstraction layer

Merge point:
- C1 and C2 must converge before build, insert, or vacuum write paths.

### Phase 4: HNSW build and query path

- [ ] Implement `FR-008` bulk build using `hnsw_rs` plus two-pass serialization.
- [ ] Implement `FR-009` scan using prepared-query scoring and `ef_search`.

Parallelism:
- Workstream D1: bulk-build heap scan, graph extraction, and TID fixup
- Workstream D2: scan-state, greedy descent, and beam search

Merge points:
- D1 depends on Phase 3.
- D2 depends on Phase 2 and Phase 3.
- End-to-end indexed query testing starts once D1 and D2 both land.

### Phase 5: Maintenance paths

- [ ] Implement `FR-010` vacuum with three-pass repair.
- [ ] Implement `FR-016` online insert and insert-drift statistics.

Parallelism:
- Workstream E1: vacuum and graph repair
- Workstream E2: online insert and metadata/statistics exposure

Merge points:
- Both depend on Phase 3.
- E2 also depends on code-to-code scoring from Phase 2.

### Phase 6: Optimization and verification

- [ ] Implement `FR-014` SIMD acceleration after scalar correctness is stable.
- [ ] Execute `NFR-001` latency/throughput benchmarks.
- [ ] Execute `NFR-002` storage accounting benchmarks.
- [ ] Execute `NFR-003` recall, ablation, drift, and vacuum-impact benchmarks.
- [ ] Harden `NFR-004` fuzzing, panic resistance, and unsafe audits.
- [ ] Enforce `NFR-005` CI gates.

Parallelism:
- Workstream F1: SIMD optimization
- Workstream F2: performance/storage benchmarking
- Workstream F3: recall/drift/quality benchmarking
- Workstream F4: safety audits and fuzzing

## Delegation / Parallel Worker Suggestions

These are the highest-value parallel slices if execution is delegated across agents or engineers:

- Worker 1: `quant/` math and `ProdQuantizer` scalar implementation
- Worker 2: datum type, text/binary I/O, and SQL registration
- Worker 3: page layout, tuple codecs, and WAL wrapper utilities
- Worker 4: HNSW bulk build and graph serialization
- Worker 5: HNSW scan and prepared-query path
- Worker 6: vacuum + online insert maintenance paths
- Worker 7: SIMD optimization and benchmark harnesses
- Worker 8: CI, fuzzing, and unsafe-audit enforcement

Preferred sequencing for parallel teams:

1. Start Workers 1, 2, 3, and 8 immediately.
2. Start Worker 5 as soon as Workers 1 and 3 have stable interfaces.
3. Start Worker 4 as soon as Worker 3 has stable tuple/page APIs.
4. Start Worker 6 after Worker 3 and Worker 1 are merged.
5. Start Worker 7 only after scalar APIs are frozen enough to optimize safely.

Task files for distribution and tracking live under `plan/tasks/`.

## Test Plan

### Module A: Datum and I/O

- [ ] `TC-001`, `TC-002`, `TC-003`: payload length and pack/unpack correctness for `FR-001`
- [ ] `TC-004`, `TC-005`, `TC-006`: text parser/formatter correctness and error handling for `FR-002`
- [ ] `TC-007`, `TC-104`: binary send/receive and truncation checks for `FR-003`
- [ ] `TC-101`, `TC-102`, `TC-103`: SQL-level type visibility and text I/O

Entrance criteria:
- Datum layout API exists.

Exit criteria:
- Round-trip and error-path tests pass at both unit and SQL levels.

### Module B: Quantizer Core and Orchestrator

- [ ] `TC-008` to `TC-015`: quantizer math, determinism, and fidelity for `FR-013`
- [ ] `TC-019` to `TC-033`: `ProdQuantizer`, encode API, pack/unpack, cache reuse, and allocation guarantees for `FR-004` and `FR-015`
- [ ] `TC-035`: random-input parser fuzz stability for `NFR-004`
- [ ] `TC-036`: unsafe comment audit for `NFR-004`

Entrance criteria:
- Scalar quantizer primitives compile.

Exit criteria:
- All scalar math and packing tests pass without panic.

### Module C: Scoring Surface

- [ ] `TC-110`, `TC-111`, `TC-129`: code-to-code and query-to-code behavioral checks for `FR-005` and `FR-017`
- [ ] `TC-134`: negative wrapper correctness for `FR-018`
- [ ] `BC-003`, `BC-010`: microbenchmarks for the scoring hot paths

Entrance criteria:
- `ProdQuantizer` scoring APIs exist and are callable from SQL bindings.

Exit criteria:
- Public scoring APIs match formulas and wrapper semantics.

### Module D: SQL Registration and Packaging

- [ ] `TC-108`, `TC-109`, `TC-114`: operators and operator class for `FR-006`
- [ ] `TC-101`, `TC-116`, `TC-130`: extension lifecycle and multi-version packaging for `FR-012`

Entrance criteria:
- SQL functions are exported.

Exit criteria:
- `CREATE EXTENSION`, operators, and pg14-pg17 support work.

### Module E: Page Layout and AM Build

- [ ] `TC-034`, `TC-117`, `TC-124`, `TC-126`, `TC-127`: page tuple correctness and page-extension/deadlock checks for `FR-007`
- [ ] `TC-112`, `TC-122`, `TC-123`, `TC-124`, `TC-125`: build-path correctness for `FR-008`
- [ ] `TC-119`: crash-recovery path for `FR-011`

Entrance criteria:
- Page structs and WAL wrappers compile.

Exit criteria:
- Build produces readable metadata and valid graph tuples.

### Module F: Scan, Vacuum, and Insert

- [ ] `TC-113`, `TC-120`, `TC-121`, `TC-131`: scan behavior for `FR-009`
- [ ] `TC-115`, `TC-118`, `TC-132`, `BC-016`: vacuum behavior and post-vacuum quality for `FR-010`
- [ ] `TC-128`, `TC-133`, `BC-011`: insert behavior and drift observability for `FR-016`

Entrance criteria:
- Bulk build and page traversal work.

Exit criteria:
- Indexed query path, vacuum, and insert are all operational and measurable.

### Module G: SIMD and Quality Verification

- [ ] `TC-016`, `TC-017`, `TC-030`, `BC-008`: SIMD correctness and throughput for `FR-014`
- [ ] `BC-001`, `BC-002`, `BC-003`, `BC-015`: latency and throughput for `NFR-001`
- [ ] `BC-004`, `BC-009`: storage and relation-size accounting for `NFR-002`
- [ ] `BC-005` to `BC-016` as applicable: recall, ablation, drift, and post-vacuum quality for `NFR-003`

Entrance criteria:
- Scalar implementation passes all correctness tests.

Exit criteria:
- Benchmark artifacts are reproducible and meet or report against the declared targets.

## Recommended Execution Order for Delegated Work

### Must land first

- [ ] Scalar quantizer math
- [ ] Datum pack/unpack and type registration
- [ ] Page layout and WAL utility layer

### Can land in parallel after the foundations are stable

- [ ] Public encode/score SQL functions
- [ ] Extension SQL packaging
- [ ] HNSW bulk build
- [ ] HNSW scan
- [ ] Safety tooling and CI checks

### Should land last

- [ ] Online insert
- [ ] SIMD optimization
- [ ] Full benchmark suite and quality gating

## Risks and Coordination Notes

- The highest merge-risk interfaces are:
  - datum binary layout
  - `ProdQuantizer` scoring signatures
  - page tuple binary layout
  - WAL wrapper APIs
- Freeze those interfaces early before parallel workers fan out too far.
- Do not optimize SIMD before scalar outputs are test-locked.
- Do not start post-vacuum and insert-drift benchmarking before build/scan semantics are stable, or benchmark churn will hide correctness issues.
