# Implementation Plan: tqvector

This plan is derived from the current `spec/` set for `tqvector`, with dependency edges inferred primarily from `traces:` frontmatter and validated against the requirement text.

Last updated: 2026-04-08 (A3 closed; A4 recall gate is next).

## Current Task Board

### Runtime lane (coder-1)

- `A1` AM split: **done**
- `A2` graph/search traversal seam: **done** (search seam extraction complete)
- `A3` wire graph-first scan runtime: **done** (cursor-owned runtime, reviews 161-193)
- `A4` recall gate: **ready to start** ← current focus
- `A5` graph-aware insert: blocked on A4
- `A6` vacuum repair: blocked on A4

### Parallel lanes

- `B1` SIMD: **in progress** (coder-2, feature branch; merge after A4 confirms scalar correctness)
- `B2` CI / fuzz / quality gates: mostly complete (TC-036 unsafe audit remaining)
- `D1` planner scaffold: **done** (merged to main from planner-integration-lane + planner-part2)
- `D2` planner activation: blocked on A4 and ADR-011 retirement

### Current sequencing

1. **Coder-1:** A4 — Recall@10 measurement over built indexes with graph-first scan. Go/no-go threshold (NFR-003).
2. **Coder-2:** B1 — SIMD acceleration (AVX2+FMA, NEON, runtime detection). Feature branch, merge after A4.
3. After A4 passes: merge SIMD, D2 planner activation, A5 insert, A6 vacuum can proceed.
4. Full SQL benchmark result generation after A5/A6.

## Requirements Summary

### Stakeholder Requirements

- [x] **StR-001**: Native compressed vector storage and ANN search inside PostgreSQL.
- [x] **StR-002**: MIT-licensed extension owned by Agent-IX with permissive dependencies only.
- [x] **StR-003**: Partition-local HNSW operation with no cross-partition coupling.

### Functional Requirements

- [x] **FR-001**: Register `tqvector` type and binary datum layout.
- [x] **FR-002**: Text I/O for `tqvector`.
- [x] **FR-003**: Binary send/receive protocol for `tqvector`.
- [x] **FR-004**: `encode_to_tqvector` API from fp32 arrays.
- [x] **FR-005**: Code-to-code inner product function.
- [x] **FR-006**: SQL operators and operator class.
- [x] **FR-007**: HNSW page layout and tuple/page invariants.
- [x] **FR-008**: HNSW bulk build callbacks.
- [ ] **FR-009**: HNSW scan callbacks and `ef_search` behavior.
- [ ] **FR-010**: HNSW vacuum callbacks.
- [x] **FR-011**: WAL safety via GenericXLog.
- [x] **FR-012**: SQL bootstrap and extension packaging.
- [x] **FR-013**: Two-stage quantization pipeline.
- [ ] **FR-014**: SIMD acceleration with scalar fallback.
- [x] **FR-015**: `ProdQuantizer` orchestrator and core encode/decode/score APIs.
- [ ] **FR-016**: HNSW online insert callbacks and insert-drift statistics.
- [x] **FR-017**: Prepared-query inner product function.
- [x] **FR-018**: Negative score wrapper functions.
- [ ] **FR-019**: Async I/O via PG18 `read_stream` API.
- [ ] **FR-020**: Planner cost estimation (`amcostestimate`, `amgettreeheight`).
- [ ] **FR-023**: Strategy translation callbacks (PG18).
- [ ] **FR-024**: Custom EXPLAIN scan diagnostics (PG18).

### Non-Functional Requirements

- [ ] **NFR-001**: Query latency and throughput targets. _Microbench infrastructure done; SQL-level blocked on scan._
- [ ] **NFR-002**: Storage compression and index-size accounting. _Layout assertions done; pg_relation_size blocked on scan._
- [ ] **NFR-003**: Recall quality and benchmark methodology. _Quantizer-level harness done (uniform + clustered + near-dup); HNSW-level blocked on scan._
- [ ] **NFR-004**: Safety and stability. _Fuzz targets (4), miri (11), proptest (15) done; unsafe audit remaining._
- [ ] **NFR-005**: Build and CI quality gates. _CI pipeline, Makefile, proptest, layout-check, bench-action done; cargo deny wired._

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
- **`FR-009 (graph traversal) -> FR-016 (graph-aware insert), FR-010 (vacuum)`**
  Reason: insert's greedy descent + beam search and vacuum's graph repair both reuse the same page-level graph traversal algorithm implemented for scan. This shared traversal helper is the critical dependency.
- **`FR-009 (validated scan + recall gate) -> FR-020 (cost estimation)`**
  Reason: realistic cost estimates require a working ordered scan. ADR-011 gates planner activation until recall is validated (A4).
- `FR-020 -> FR-023, FR-024`
  Reason: strategy translation and custom EXPLAIN only matter once the planner can select the index.
- `FR-009 -> FR-019`
  Reason: async I/O replaces synchronous buffer reads in scan hot path; needs working scan first.
- `FR-012`
  Reason: bootstrap/packaging is mostly independent, but final SQL registration depends on available functions and operators.
- `FR-014`
  Reason: can start after scalar quantizer/scoring APIs exist; not a blocker for correctness-first implementation.

### Cross-cutting constraints

- `NFR-004` applies to all parser, page, WAL, and unsafe code paths.
- `NFR-005` applies to every merge point.
- `NFR-001`, `NFR-002`, and `NFR-003` are verification-heavy and should not block initial correctness unless a design choice depends on benchmark evidence.

---

## Completed Phases

Phases 0-3 and the build half of Phase 4 are complete. Preserving the record here for traceability; active work is described in the Remaining Work section below.

### Phase 0: Scaffolding — COMPLETE

- [x] Crate/module skeleton (type, quantizer, SQL functions, AM modules)
- [ ] CI skeleton — Makefile targets exist, CI pipeline not wired (tracked in Task 08)

### Phase 1: Scalar quantizer core — COMPLETE

- [x] `FR-013` scalar math (SRHT/FWHT, codebook, MSE/QJL packing)
- [x] `FR-015` scalar `ProdQuantizer` (constructor, encode/decode, query prep, scoring, cache)

### Phase 2: Datum and SQL function surface — COMPLETE

- [x] `FR-001` datum pack/unpack and type registration
- [x] `FR-002` text I/O
- [x] `FR-003` binary send/receive
- [x] `FR-004` `encode_to_tqvector`
- [x] `FR-005`, `FR-017`, `FR-018` SQL-visible scoring functions
- [x] `FR-006` operators and operator class
- [x] `FR-012` extension packaging

### Phase 3: Page layout and storage engine base — COMPLETE

- [x] `FR-007` page structs, tuple codecs, fit checks, level-cap logic
- [x] `FR-011` GenericXLog wrapper utilities and write discipline

### Phase 4 (partial): HNSW bulk build — COMPLETE

- [x] `FR-008` bulk build using `hnsw_rs` plus two-pass serialization
- [x] Scan lifecycle, query validation, metadata/prepared-query caching, bootstrap linear scan
- [x] Insert shape validation, metadata init, duplicate coalescing, tail-page append/reuse
- [x] AM module split: cost, vacuum, options, routine, build extracted from mod.rs

---

## Remaining Work

### Current State Summary

The extension is ~70% complete. All quantizer, type, scoring, page layout, and build code is done.
The critical gap is FR-009 ordered graph traversal scan — without it, index queries don't work.
Planner/config groundwork is now substantially complete on `main`: the pure cost model, callback
scaffolding, and `ef_search` control-surface wiring are merged, while planner-visible scans remain
intentionally disabled behind ADR-011 and D2 remains blocked on A4. Insert and vacuum also need
graph traversal for neighbor selection and graph repair, making scan the single gating dependency.

### Remaining Dependency Graph

```
                    ┌─────────────────────────┐
                    │  A1: Finish am split     │
                    │  (complete)              │
                    └───────────┬──────────────┘
                                │
                    ┌───────────▼──────────────┐
                    │  A2: Graph/search        │
                    │  traversal seam          │
                    │  (substantially complete)│
                    └───────────┬──────────────┘
                                │
              ┌─────────────────┼─────────────────┐
              │                 │                  │
  ┌───────────▼────────┐  ┌────▼───────────┐  ┌──▼──────────────────┐
  │  A3: Wire scan      │  │                │  │                     │
  │  (amgettuple +      │  │  A5: Graph     │  │  A6: Vacuum         │
  │  ef_search GUC)     │  │  insert        │  │  three-pass         │
  │                     │  │  (FR-016)      │  │  (FR-010)           │
  └───────────┬─────────┘  └────────────────┘  └─────────────────────┘
              │                    │                      │
  ┌───────────▼─────────┐         │                      │
  │  A4: Recall gate    │         │                      │
  │  (measure recall,   │◄────────┘──────────────────────┘
  │  go/no-go before    │    (drift/vacuum recall needs A5/A6)
  │  proceeding)        │
  └───────────┬─────────┘
              │
              │ GATES
              ▼
  ┌───────────────────────────────────────────┐
  │  D2: Wire planner (FR-020 costs,         │
  │  remove ADR-011, FR-023, FR-024, FR-019) │
  └───────────────────────────────────────────┘


  PARALLEL (no dependencies on Track A):

  ┌─────────────────────┐    ┌─────────────────────┐    ┌─────────────────────────┐
  │  B1: SIMD (FR-014)  │    │  B2: CI (NFR-005)   │    │  D1: Planner scaffold   │
  │  AVX2+FMA, NEON,    │    │  + fuzz (NFR-004)   │    │  (cost model tests,     │
  │  runtime detection  │    │                     │    │  strategy stubs,        │
  └─────────────────────┘    └─────────────────────┘    │  EXPLAIN skeleton)      │
                                                        └─────────────────────────┘
```

### Track A: Critical Path (serial, primary agent)

All items are serial because each depends on the previous.

#### A1: Finish am/mod.rs Split
- **Scope:** Extract insert and scan callbacks into `am/insert.rs` and `am/scan.rs`. Mechanical refactor, no logic changes.
- **Owns:** Structural prep for FR-009, FR-016
- **Estimated new code:** ~100 lines (visibility adjustments)
- **Difficulty:** Easy
- **Exit criteria:** am/mod.rs contains only shared helpers (metadata, page utils, build callback). Each AM concern lives in its own file.
- **Status:** Complete

#### A2: Graph Traversal Helpers
- **Scope:** Finish the shared page-level traversal seam: layer-0 neighbor slicing, page-level
  greedy/beam traversal, and the ownership boundary between traversal state and scan scoring
  inputs. Both scan (FR-009) and insert (FR-016) need this same algorithm — scan uses LUT
  scoring (`score_ip_encoded`), insert uses code-to-code scoring (`score_ip_codes_lite`).
- **Owns:** Shared foundation for FR-009, FR-016, FR-010
- **Estimated new code:** ~250-350 lines
- **Difficulty:** Hard — this is the core HNSW algorithm on Postgres buffer pages
- **Key challenges:**
  - Buffer pin management: read page, decode neighbor tuple, follow TID pointers, release buffer per tuple
  - Visited set: HashSet of ItemPointer, potentially large for big indexes
  - BinaryHeap ordering: min-heap for candidates (closest first), correct handling of negative inner product direction
  - Layer traversal: read neighbor TIDs at correct layer offset within TqNeighborTuple
- **Exit criteria:** Graph owns layer-0 traversal helpers and search owns the bootstrap
  visible-frontier protocol around them, leaving `scan.rs` with only the runtime shell, scan-owned
  sets, and result adjudication.
- **Status:** Substantially complete. Remaining work belongs to A3, not additional seam extraction.
- **Reference:** pgvector `hnsw_search_layer` in `hnswscan.c` (~150 lines), but tqvector needs raw page decode which adds complexity.

#### A3: Wire Graph Traversal into Scan
- **Scope:** Make graph/search traversal the primary scan execution path. Consume the resolved
  `ef_search` control surface (session override over reloption). Keep the current linear path as
  the fallback shell for empty/tiny indexes and bootstrap exhaustion. Replace MAX cost estimates
  with realistic planner costs only after the ordered execution contract is credible.
- **Owns:** FR-009 completion
- **Estimated new code:** ~200-300 lines
- **Difficulty:** Medium — wiring + GUC registration + planner integration
- **Key deliverables:**
  1. `amgettuple` consumes the graph/search-owned bootstrap traversal path as the primary ordered
     runtime source.
  2. The linear path remains the explicit fallback shell rather than the default execution lane.
  3. `tqhnsw.ef_search` runtime wiring stays authoritative for bootstrap frontier sizing.
  4. Duplicate heap TID handling for coalesced element tuples remains correct under graph-first
     execution.
- **Exit criteria:** `SET enable_seqscan = off; SELECT ... ORDER BY col <#> $query LIMIT 10` returns distance-ordered results via index scan. ADR-011 cost gate remains active.
- **Status: CLOSED** (2026-04-08, reviews 161-193). Graph-first scan runtime is cursor-owned end-to-end. Bootstrap helpers gated to test/debug.
- **Post-v0.1 follow-up items:**
  1. **Raw pointer aliasing in `GraphTraversalPrefetchContext::run`** — The `self as *mut Self` pattern is sound but fragile. Consider replacing the closure bundle in `select_next_with_refill` with a single trait object or visitor to eliminate self-aliasing.
  2. **Graph exhaustion → fallback transition policy** — Currently graph exhaustion always marks `Exhausted`. Partial-graph indexes (sparse connectivity, live-write connectivity gaps after A5) may benefit from a graceful transition to `LinearFallback` instead. Evaluate after A5 introduces indexes with connectivity gaps.
  3. **`with_visible_frontier_mut_and_bootstrap_expansion` borrow splitting** — Exists to work around disjoint field access through `TqScanOpaque`. If graph-phase state is refactored into its own struct (cursor extraction laid the groundwork), this helper can be replaced with direct field borrows.
  4. **`GraphTraversalCursor` reconstruction per call** — Cursor is rebuilt via `graph_traversal_cursor(opaque)` multiple times per `amgettuple` call. Likely inlined away by the compiler, but holding the cursor across the full `produce_next_graph_traversal_heap_tid` body would read more naturally.
  5. **Integration-level ordered-result regression test** — Cursor mechanics have strong unit coverage, but no integration test confirms graph-first heap TIDs arrive in distance-sorted order across a non-trivial built index. Should come naturally from A4 recall measurement — persist as a regression test afterward.

#### A4: Recall Benchmark Gate
- **Scope:** Measure Recall@10 on synthetic data after ordered result buffering is in place. This
  is a go/no-go gate — if recall is below ~89% (m=8, ef=128 per NFR-003), stop and investigate
  before investing in insert/vacuum.
- **Owns:** Initial NFR-003 validation
- **Estimated new code:** ~200-300 lines (benchmark harness + ground truth generator)
- **Difficulty:** Easy-Medium
- **Key deliverables:**
  1. Brute-force fp32 top-k ground truth generator
  2. Recall@10 measurement (set intersection)
  3. Test at multiple configurations: (m=8, ef=40), (m=8, ef=128), (m=8, ef=200), (m=16, ef=200)
  4. Report recall numbers — these anchor all downstream quality claims
- **Exit criteria:** Recall@10 >= 89% at m=8 ef=128 on 10K+ vectors at 1536-dim 4-bit. If not met, investigate root cause before proceeding.

#### A5: Graph-Aware Insert (FR-016)
- **Scope:** Replace disconnected-append insert with graph-connected insert. Layer assignment, greedy descent via A2 helpers, beam search for neighbors, back-link updates, entry point promotion, drift statistics.
- **Owns:** FR-016 completion
- **Estimated new code:** ~400-600 lines
- **Difficulty:** Hard
- **Key challenges:**
  - Lock ordering across multiple page writes (back-links touch many pages per insert)
  - Pruning weakest neighbor when at capacity M (needs scoring of existing neighbors)
  - Concurrency: two concurrent inserts touching overlapping neighbor lists
  - Entry point update when new node has higher layer than current max
  - Drift statistics: `inserted_since_rebuild` counter in metadata
- **Dependencies:** A2 (traversal helpers), A3 (working scan for testing reachability)
- **Exit criteria:** Inserted rows are reachable via HNSW scan. No deadlock under concurrent insert. Drift counter is queryable.

#### A6: Vacuum Three-Pass (FR-010)
- **Scope:** Implement ambulkdelete with mark/repair/finalize passes. Graph repair uses A2 traversal to find replacement neighbors. amvacuumcleanup updates pg_class stats.
- **Owns:** FR-010 completion
- **Estimated new code:** ~400-600 lines
- **Difficulty:** Hard — graph repair while maintaining connectivity is the hardest correctness problem
- **Key challenges:**
  - Pass 2 graph repair: for each broken connection, search for replacement neighbors using A2 traversal with code-to-code scoring
  - Concurrency with ongoing inserts and scans during vacuum
  - Maintaining recall >= 80% of pre-vacuum after 10% deletion (FR-010-AC-2)
  - GenericXLog for all page modifications during repair
- **Dependencies:** A2 (traversal helpers), A5 (neighbor selection/pruning logic is shared with insert)
- **Exit criteria:** Deleted rows absent from results. Recall maintained. No corruption under concurrent load.

### Track B: Parallel Work (independent agent, can start immediately)

These items have no dependency on Track A. They depend only on the frozen scalar APIs from completed phases.

#### B1: SIMD Acceleration (FR-014)
- **Scope:** AVX2+FMA and NEON implementations of `fwht`, `score_ip_encoded`, `score_ip_encoded_lite`, `qjl_bit_expand`. Runtime feature detection. Scalar fallback.
- **Owns:** FR-014
- **Estimated new code:** ~700-900 lines
- **Difficulty:** Medium
- **Key challenges:**
  - FWHT butterfly pattern in AVX2 (register shuffles across 256-bit lanes)
  - Variable bit-width MSE index unpacking in SIMD (bits range 2-8)
  - Testing on both x86_64 and aarch64
- **Dependencies:** None — scalar APIs are frozen
- **Exit criteria:** SIMD-scalar equivalence within 1e-6 on 1000 random inputs. fwht AVX2 >= 3x scalar throughput. No SIGILL on unsupported CPU.

#### B2: CI Pipeline and Safety (NFR-004, NFR-005) — MOSTLY COMPLETE
- **Scope:** Wire Makefile targets to GitHub Actions. Add fuzz harness for `tqvector_in`. Audit unsafe blocks for SAFETY comments. `cargo deny` in CI.
- **Owns:** NFR-004, NFR-005
- **Actual code:** CI YAML (57 new lines), 4 fuzz targets, 11 miri tests, 15 property tests, 13 layout assertions, clippy.toml, 18 Makefile targets
- **Difficulty:** Easy
- **Dependencies:** None
- **Exit criteria:** PR merges require passing fmt, clippy, test, pgrx test, deny. Fuzz harness runs 10K random byte sequences without panic.
- **Status:** All done except TC-036 (formal unsafe block audit). CI pipeline, fuzz, miri, proptest, layout-check, bench-action all landed.

### Track D: Planner Integration (independent agent, partially parallel)

Planner integration spans two phases: scaffolding that can start now, and wiring that is gated on the recall gate (A4).

#### D1: Planner Scaffolding (can start now)
- **Scope:** Build the cost model, strategy translation, custom EXPLAIN, and async I/O scaffolding behind PG-version feature gates. All code compiles and is testable in isolation but is not activated in `amcostestimate` (ADR-011 gate remains).
- **Owns:** FR-020 (partial), FR-023, FR-024, FR-019 (scaffolding only)
- **Estimated new code:** ~400-500 lines
- **Difficulty:** Medium
- **Key deliverables:**
  1. Cost model function computing startup/total cost from metadata (m, ef_search, dimensions, max_level, index_pages, reltuples) — unit-testable without a running index
  2. `amgettreeheight` callback reading max_level from metadata page (PG18, feature-gated)
  3. `amtranslatestrategy` / `amtranslatecmptype` stubs returning `COMPARE_LT` for strategy 1 (PG18, feature-gated)
  4. `TqScanOpaque` counter struct contract for custom EXPLAIN (`TqExplainCounters`) plus pure output/gating helpers
  5. EXPLAIN hook skeleton that reads counters and emits ExplainProperty calls (PG18, feature-gated)
  6. ReadStream callback signatures for graph and linear streams (PG18, feature-gated, not yet wired into scan loop)
  7. Unit tests for cost model: planner selects index at 10K rows, prefers seqscan at 50 rows, handles edge cases (empty index, zero reltuples)
- **File ownership:** `am/cost.rs` (cost model + amgettreeheight), `am/explain.rs` (EXPLAIN hook), `am/stream.rs` (async I/O). These files do not overlap with graph search agent's `am/scan.rs` and `am/search.rs`.
- **Status:** substantially complete on `main` after merging `planner-integration-lane` and `planner-part2`; only D2 wiring remains blocked on A4.
- **Exit criteria:** All scaffolding compiles, tests pass, but `amcostestimate` still returns `f64::MAX`. No functional change to query behavior.

#### D2: Wire Planner (gated on A4 recall gate)
- **Scope:** Replace ADR-011 `f64::MAX` override with the real cost model from D1. Wire async I/O streams into scan loop. Activate EXPLAIN counters. Mark ADR-011 as superseded.
- **Owns:** FR-020 (activation), FR-019 (activation), FR-024 (activation)
- **Estimated new code:** ~100-150 lines (wiring, not new logic)
- **Difficulty:** Easy-Medium — connecting scaffolding to live paths
- **Key deliverables:**
  1. `amcostestimate` calls real cost model function instead of returning `f64::MAX`
  2. ReadStream instances created in `amrescan`, used in scan loop, destroyed in `amendscan`
  3. EXPLAIN counters incremented during scan execution
  4. ADR-011 marked superseded
  5. FR-020-AC-1 validated: EXPLAIN shows index scan on 10K-row table
  6. FR-020-AC-2 validated: planner prefers seqscan on 50-row table
- **Dependencies:** A4 (recall gate must pass), D1 (scaffolding must be complete)
- **Exit criteria:** `SELECT ... ORDER BY col <#> $query LIMIT 10` uses index scan without `enable_seqscan = off`. EXPLAIN (tqvector) shows scan stats on PG18.

### Track C: Post-Gate Verification (after A4 passes)

These items only make sense after graph scan is validated and recall is confirmed.

#### C1: Full Benchmark Suite — INFRASTRUCTURE COMPLETE
- **Scope:** NFR-001 latency/throughput benchmarks, NFR-002 storage accounting, NFR-003 full recall suite (ablation, drift after inserts, post-vacuum quality).
- **Owns:** NFR-001, NFR-002, NFR-003 (full)
- **Actual code:** ~2700 lines across 35 files (8 criterion suites, 3 iai suites, 2 dhat bins, 4 fuzz targets, 15 property tests, 13 layout assertions, recall harness with 3 data distributions, SQL scripts, CI integration, reporting template)
- **Difficulty:** Medium
- **Dependencies:** A3 (scan works), A4 (recall gate passed), A5 (insert drift), A6 (vacuum quality)
- **Status:** All benchmark infrastructure is built and validated. Quantizer-level benchmarks (BC-010, BC-017 to BC-031) can run now. SQL-level benchmarks (BC-001 to BC-009, BC-011 to BC-016) remain blocked on working scan/insert/vacuum.
- **Exit criteria:** Reproducible benchmark artifacts meeting or reporting against declared targets.

---

## Parallel Execution Summary

```
Time ──────────────────────────────────────────────────────────────────────────────────►
                                    ▼ NOW

Coder-1 (graph search — critical path):
  [A1: split] → [A2: traversal] → [A3: scan ~~~~~~~~] → [A4: recall gate] → [A5: insert] → [A6: vacuum] → [C1: benchmarks]
   DONE          DONE               IN PROGRESS          ~1 sess             ~2-3 sess      ~2-3 sess      ~1-2 sess

Coder-2 (planner → SIMD):
  [D1: scaffold ~~~~~~~~] → [B1: SIMD ~~~~~~~~~~~~~~] → [D2: wire planner] → done
   DONE (merged to main)    IN PROGRESS (feature branch)  ~1 session
                                        │                       │
                                        ← merge after A3   BLOCKED until A4
```

**Minimum viable extension** (index queries work): A1 + A2 + A3 + A4 = **5-7 sessions**
**Full v0.1 completion**: all tracks = **~12-18 sessions** total wall time

---

## Task File Mapping

| Task File | Covers | Status |
|-----------|--------|--------|
| `01-quantizer-core.md` | Phase 1 | complete |
| `02-datum-and-io.md` | Phase 2 (type/I/O) | complete |
| `03-sql-surface.md` | Phase 2 (functions/operators) | complete |
| `04-page-layout-and-wal.md` | Phase 3 | complete |
| `05-graph-scan.md` | A1 + A2 + A3 + A4 (scan critical path) | **A1/A2 done, A3 in progress (coder-1)** |
| `06-graph-insert.md` | A5 (graph-aware insert) | blocked on 05 |
| `07-vacuum.md` | A6 (vacuum three-pass) | blocked on 05, 06 |
| `08-simd.md` | B1 (SIMD acceleration) | **in progress (coder-2, feature branch)** |
| `09-ci-and-safety.md` | B2 (CI pipeline, fuzz, audit) | mostly complete (unsafe audit remaining) |
| `10-benchmarks.md` | C1 (full benchmark suite) | **infrastructure complete**, NFR runs blocked on 05 |
| `11-planner.md` | D1 + D2 (planner integration) | **D1 done on main; D2 blocked on A4** |

---

## Test Plan

### Module A: Datum and I/O — COMPLETE

- [x] `TC-001`, `TC-002`, `TC-003`: payload length and pack/unpack correctness for `FR-001`
- [x] `TC-004`, `TC-005`, `TC-006`: text parser/formatter correctness and error handling for `FR-002`
- [x] `TC-007`, `TC-104`: binary send/receive and truncation checks for `FR-003`
- [x] `TC-101`, `TC-102`, `TC-103`: SQL-level type visibility and text I/O

### Module B: Quantizer Core and Orchestrator — COMPLETE

- [x] `TC-008` to `TC-015`: quantizer math, determinism, and fidelity for `FR-013`
- [x] `TC-019` to `TC-033`: `ProdQuantizer`, encode API, pack/unpack, cache reuse, and allocation guarantees for `FR-004` and `FR-015`
- [x] `PT-001` to `PT-010`: property tests for SRHT, pack/unpack, determinism, symmetry, score consistency, decode error bound
- [x] `MI-001` to `MI-007`: miri UB tests for encode, pack/unpack, scoring, hadamard
- [x] `FZ-001` to `FZ-002`: fuzz targets for parse_text and unpack_mse
- [x] `TC-035`: random-input parser fuzz stability for `NFR-004`
- [ ] `TC-036`: unsafe comment audit for `NFR-004`

### Module C: Scoring Surface — COMPLETE

- [x] `TC-110`, `TC-111`, `TC-129`: code-to-code and query-to-code behavioral checks for `FR-005` and `FR-017`
- [x] `TC-134`: negative wrapper correctness for `FR-018`
- [x] `BC-010`: score_ip_encoded throughput measured (~95K scores/sec at 1536/4-bit)
- [x] `BC-024` to `BC-026`: score_ip_from_parts, score_ip_encoded_lite, decode_approximate throughput
- [x] `BC-027` to `BC-029`: iai-callgrind instruction count benchmarks (scoring, hadamard, bitpack)
- [x] `BC-030`, `BC-031`: dhat heap profiling harnesses (scoring zero-alloc, encode profile)
- [ ] `BC-003`: SQL-level scoring latency (blocked on scan)

### Module D: SQL Registration and Packaging — COMPLETE

- [x] `TC-108`, `TC-109`, `TC-114`: operators and operator class for `FR-006`
- [x] `TC-101`, `TC-116`, `TC-130`: extension lifecycle and multi-version packaging for `FR-012`

### Module E: Page Layout and AM Build — COMPLETE

- [x] `TC-034`, `TC-117`, `TC-124`, `TC-126`, `TC-127`: page tuple correctness and page-extension/deadlock checks for `FR-007`
- [x] `TC-112`, `TC-122`, `TC-123`, `TC-124`, `TC-125`: build-path correctness for `FR-008`
- [x] `TC-119`: crash-recovery path for `FR-011`
- [x] `PT-011` to `PT-015`: property tests for page tuple roundtrips, encoded_len
- [x] `MI-008` to `MI-011`: miri UB tests for ItemPointer, element/neighbor/metadata tuples
- [x] `FZ-003`, `FZ-004`: fuzz targets for element_tuple_decode and neighbor_tuple_decode
- [x] `LA-001` to `LA-013`: size_of layout assertions (payload sizes, struct sizes, compression ratio)
- [x] `BC-022`, `BC-023`: DataPage insert/read element and neighbor throughput

### Module F: Scan, Vacuum, and Insert — IN PROGRESS

- [ ] `TC-113`, `TC-120`, `TC-121`, `TC-131`: scan behavior for `FR-009`
- [ ] `TC-115`, `TC-118`, `TC-132`, `BC-016`: vacuum behavior and post-vacuum quality for `FR-010`
- [ ] `TC-128`, `TC-133`, `BC-011`: insert behavior and drift observability for `FR-016`

Entrance criteria:
- Graph traversal helpers work on built indexes.

Exit criteria:
- Indexed query path, vacuum, and insert are all operational and measurable.

### Module I: Planner Integration — NOT STARTED

- [ ] Cost model unit tests: planner selects index at 10K rows, prefers seqscan at 50 rows, handles edge cases for `FR-020`
- [ ] `FR-020-AC-1`: EXPLAIN shows index scan on 10K-row table
- [ ] `FR-020-AC-2`: planner prefers seqscan on 50-row table
- [ ] `FR-020-AC-3`: cost model reads metadata (m, ef_search, dimensions, max_level)
- [ ] `FR-020-AC-4`: `amgettreeheight` returns max_level on PG18
- [ ] `FR-020-AC-5`: ADR-011 marked superseded
- [ ] Strategy translation: `amtranslatestrategy` returns `COMPARE_LT` for strategy 1, `COMPARE_INVALID` otherwise for `FR-023`
- [ ] Custom EXPLAIN: `EXPLAIN (tqvector)` shows scan counters on PG18 for `FR-024`
- [ ] Async I/O: ReadStream prefetch improves cold-cache scan latency on PG18 for `FR-019`

Entrance criteria:
- D1 scaffolding: none (can start now with unit tests against cost model function)
- D2 wiring: A4 recall gate passed, graph search agent no longer modifying `am/scan.rs`

Exit criteria:
- Planner naturally selects tqhnsw index on large tables without `enable_seqscan = off`.

### Module G: Benchmark Infrastructure — COMPLETE

- [x] Criterion microbenchmarks: 8 suites covering all scoring variants, encode, prepare, hadamard, codebook, bitpack, page_codec, text_io
- [x] iai-callgrind: 3 suites (scoring, hadamard, bitpack) for deterministic CI regression detection
- [x] dhat heap profiling: 2 binaries (encode path, scoring zero-allocation verification)
- [x] Quantizer-level recall harness: uniform (50K) + clustered (10K/50 clusters) + near-duplicate stress test
- [x] `BC-017` to `BC-021`: quantizer recall at multiple distributions, bitwidths, dimensions
- [x] `BC-022` to `BC-026`: DataPage and scoring variant throughput
- [x] `BC-027` to `BC-031`: instruction count regression + heap profiling harnesses
- [x] Makefile targets (18), CI pipeline (benchmark-action with 110% threshold)
- [x] BENCHMARKS.md reporting template

### Module H: SIMD and SQL-Level Verification — NOT STARTED

- [ ] `TC-016`, `TC-017`, `TC-030`, `BC-008`: SIMD correctness and throughput for `FR-014`
- [ ] `BC-001`, `BC-002`, `BC-003`, `BC-015`: SQL-level latency and throughput for `NFR-001`
- [ ] `BC-004`, `BC-009`: storage and relation-size accounting for `NFR-002`
- [ ] `BC-005` to `BC-007`, `BC-011` to `BC-016`: HNSW recall, ablation, drift, post-vacuum quality for `NFR-003`

Entrance criteria:
- Scalar implementation passes all correctness tests.
- Graph scan returns distance-ordered results.

Exit criteria:
- Benchmark artifacts are reproducible and meet or report against the declared targets.

---

## Risks and Coordination Notes

### Frozen interfaces (no further changes expected)
- datum binary layout
- `ProdQuantizer` scoring signatures (`score_ip_encoded`, `score_ip_codes_lite`, `prepare_ip_query`)
- page tuple binary layout (`TqElementTuple`, `TqNeighborTuple`)
- WAL wrapper APIs (`GenericXLogTxn`)

### Active risks
- **Graph traversal correctness:** The A2 traversal helper is the foundation for scan, insert, and vacuum. A bug here propagates everywhere. Invest in thorough testing with known-graph topologies before building on top.
- **Buffer pin discipline:** Traversal must read/decode/release per page. Holding pins across neighbor hops will deadlock or exhaust shared_buffers. Test with `max_buffer_pins` instrumentation.
- **Recall uncertainty:** Until A4 measures recall, there is no evidence the quantizer+HNSW combination meets spec targets. This is an existential risk. Do not skip the recall gate.
- **Insert lock ordering:** Back-link updates touch multiple pages per insert. Without a consistent lock ordering protocol, concurrent inserts can deadlock. Define and document the protocol in A5 before implementing.
- **SIMD merge timing:** SIMD work (B1) can proceed in parallel but should not merge into main until after A3 confirms scalar scan correctness. Merging SIMD earlier risks masking scalar bugs behind SIMD-specific behavior.
