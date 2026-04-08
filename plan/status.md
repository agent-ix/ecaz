# Project Status

Last updated: 2026-04-07
Basis: current `main` at `15cc242` plus local planning updates in progress

## Reading Guide

- Percentages are judgment-based delivery estimates, not LOC metrics.
- `100%` means the intended v0.1 scope for that row is merged and validated on `main`.
- Infrastructure and harness completion does not count as benchmark, profiling, or optimization completion by itself.
- Rollups are weighted by delivery significance, not simple averages.

## Rollup

| Rollup | % Done | Meaning |
| --- | ---: | --- |
| Correctness-complete | 72% | Foundation/build is solid; graph-first scan, graph-aware insert, and vacuum repair still block full correctness |
| Test/validation-complete | 76% | Broad unit/integration/CI coverage exists, but graph-first scan validation and final unsafe hardening remain |
| Benchmark/profile-complete | 36% | Benchmark harnesses exist, but end-to-end HNSW latency, storage, and recall evidence is still mostly blocked |
| Optimization-complete | 12% | Scalar implementations exist, but there has not been a real optimization pass yet and SIMD is still unstarted |
| Release-ready | 54% | Build packaging and quality infrastructure are in decent shape, but runtime proof, recall signoff, and planner activation remain |
| Total project completion | 67% | Weighted overall estimate to final intended scope |

## Execution Task Map

| ID | Task | Includes | Status | % Done | Notes |
| --- | --- | --- | --- | ---: | --- |
| `A1` | AM split | `scan`, `insert`, `build`, `options`, `cost`, `vacuum`, `routine`, `shared`, `search` module split | Done | 100% | Complete on `main` |
| `A2` | Graph/search traversal seam | Layer-0 traversal helpers, visible frontier protocol, bootstrap traversal boundary | Substantially complete | 95% | Runtime seam extraction is effectively done |
| `A3` | Graph-first scan runtime | Make graph/search traversal the primary ordered scan path with linear fallback shell | Next | 20% | Main runtime blocker |
| `A4` | Recall gate | HNSW Recall@10 measurement and go/no-go threshold | Not started | 0% | Blocked on `A3` |
| `A5` | Graph-aware insert | Greedy descent, neighbor selection, backlinks, drift handling | Not started | 0% | Blocked on `A3`/`A4` |
| `A6` | Vacuum repair | Mark/repair/finalize vacuum with graph repair | Not started | 0% | Blocked on `A3`/`A4` |
| `B1` | SIMD | AVX2+FMA, NEON, runtime detection, equivalence tests, throughput proof | Not started | 0% | FR-014 still open |
| `B2` | CI / safety / quality | CI wiring, fuzz, miri, deny, layout checks, broader NFR-005 hardening | In progress | 75% | Strong base is landed; final hardening remains |
| `C1` | Full benchmark suite | NFR-001/002/003 scripts, harnesses, reporting, end-to-end result artifacts | In progress | 45% | Infrastructure is built; final result runs are blocked on `A3`/`A5`/`A6` |
| `D1` | Planner scaffold | Cost-model scaffolding, explain/stat surfaces, PG18 read-stream scaffolding | In progress | 70% | Separate lane |
| `D2` | Planner activation | Real planner enablement, credible cost model, ADR-011 retirement, PG18 scan integration | Not started | 5% | Blocked on `A4` and planner gate retirement |

## 1. Foundation / Build

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Quantizer core | Datum type, encode/decode, scoring, negative wrappers | Done | 100% | Core math and SQL-visible functions are complete |
| Storage engine base | Page layout, tuple codecs, WAL discipline, metadata model | Done | 100% | Stable storage foundation is in place |
| Bulk build path | HNSW bulk build, duplicate coalescing, metadata initialization | Done | 100% | Built indexes are valid on `main` |
| AM structure | Access method split into dedicated modules | Done | 100% | Structural groundwork is complete |

Foundation / build rollup: 100%

## 2. Scan Runtime

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Bootstrap traversal seam | Graph/search ownership split, visible frontier protocol, graph-owned layer-0 traversal helpers | Substantially complete | 95% | A2 is effectively complete |
| Graph-first ordered execution | Make graph/search traversal primary in `amgettuple` | Next | 20% | This is A3 and the main runtime blocker |
| Linear fallback policy | Keep linear scan as explicit fallback shell during A3 | In progress | 70% | Fallback exists; final runtime contract is still being defined |
| `ef_search` runtime behavior | Resolved `ef_search` drives bootstrap frontier sizing | Mostly done | 85% | Main runtime wiring landed; sentinel cleanup remains elsewhere |
| Recall gate readiness | Runtime integrity sufficient to measure HNSW Recall@10 | Blocked | 10% | Waiting on graph-first ordered execution |

Scan runtime rollup: 58%

## 3. Insert Path

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Current insert correctness | Shape validation, metadata setup, duplicate handling, tail-page append/reuse | Partial | 55% | Safe append path exists |
| Graph-aware insert | Greedy descent, neighbor selection, backlink repair | Not started | 0% | Blocked on validated graph-first scan |
| Insert drift accounting | Inserted-since-rebuild tracking and drift-aware measurement | Not started | 0% | Follows graph-aware insert |

Insert path rollup: 22%

## 4. Vacuum / Repair

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Vacuum callback scaffold | Vacuum module split and base callback structure | Partial | 35% | Structural base exists |
| Graph repair logic | Mark, repair, finalize passes over deleted nodes | Not started | 0% | Blocked on graph traversal confidence |
| Stats cleanup integration | `amvacuumcleanup` and stats alignment | Not started | 0% | Follows repair implementation |

Vacuum / repair rollup: 12%

## 5. Planner / PG18

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Planner scaffold | Cost/explain/stat/read-stream scaffolding | In progress | 70% | Separate lane has made real progress |
| Planner activation | Real index selection and credible cost model | Not started | 5% | Gated on runtime/recall |
| PG18 async/read_stream integration | Runtime scan integration with PG18 path | Not started | 10% | Scaffold exists; production integration waits on scan |
| Strategy / EXPLAIN surfaces | FR-023 / FR-024 surfaces | Partial | 45% | Descriptive surfaces exist; activation still gated |

Planner / PG18 rollup: 38%

## 6. Testing / Validation

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Unit / property / layout tests | Scalar, page, codec, search protocol, size/layout checks | Strong | 92% | Broad low-level coverage exists |
| `cargo test` / `pgrx test` integration | Extension-level build and runtime integration | Strong | 82% | Good staged-behavior coverage exists |
| CI / safety tooling | Clippy, deny, fuzz, miri, benchmark-action, nightly checks | Strong | 75% | Base infrastructure is present |
| Graph-first runtime validation | Ordered scan behavior under A3 | Not started | 15% | Needs graph-first runtime path |
| Unsafe/stability audit | Final unsafe review and hardening pass | Partial | 50% | Tooling exists; final audit remains |

Testing / validation rollup: 76%

## 7. Benchmarking / Profiling

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Microbenchmark infrastructure | Criterion, iai-callgrind, dhat, Makefile targets, shared generators | Done | 100% | Harnesses are built and validated |
| Quantizer-level benchmark runs | Pure-Rust microbench and recall-smoke evidence | Strong | 80% | Useful baseline numbers exist |
| SQL benchmark infrastructure | `bench_sql_latency.sh`, `bench_storage.sh`, `bench_recall.py`, reporting template | Done | 90% | Scripts exist, but depend on working scan/insert/vacuum |
| End-to-end HNSW latency/storage results | NFR-001 and NFR-002 result artifacts | Not started | 0% | Blocked on A3 |
| End-to-end HNSW recall results | NFR-003 result artifacts over built indexes | Not started | 0% | Blocked on A3, A5, A6 |
| Runtime hot-path profiling | Real graph traversal profiling and bottleneck evidence | Not started | 10% | Premature before graph-first scan is primary |

Benchmarking / profiling rollup: 36%

## 8. Optimization / SIMD

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Scalar baseline | Working scalar quantizer and scan code paths | Partial | 55% | Correct baseline exists, but this is not the same as an optimization pass |
| Quantizer optimization passes | Deliberate score/encode/hadamard improvement work based on profiling | Not started | 10% | Benchmark harnesses exist, but no serious optimization campaign has been run yet |
| SIMD acceleration | AVX2+FMA, NEON, runtime detection, equivalence proof, throughput proof | Not started | 0% | FR-014 remains open |
| Runtime scan optimization | Tuning the graph-first scan hot path | Not started | 0% | Wait until A3 lands |
| Memory / buffer tuning | Traversal footprint, buffer behavior, allocator-pressure tuning | Not started | 5% | Some design notes exist, not a real tuning pass yet |

Optimization / SIMD rollup: 12%

## 9. Docs / Specs / Coordination

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Spec / ADR corpus | FR/NFR/ADR coverage and decision boundaries | Strong | 85% | Good structure already exists |
| Task planning surface | Project plan and task docs | Improving | 78% | Mostly solid; still being tightened for scanability |
| Review memory | Review request and feedback trail | Strong | 92% | Detailed history exists in `review/*` |
| Status reporting | Stable project-level status snapshot | In progress | 60% | This file is intended to make status easier to read |

Docs / coordination rollup: 80%

## 10. Release / CI / Quality Gates

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Extension packaging/build | Extension packaging and local build/install path | Done | 100% | Solid |
| Quality gate infrastructure | CI, deny, miri, fuzz, benchmark-action, validation commands | Strong | 75% | Good base exists |
| Release proof | Recall signoff, planner/runtime readiness, benchmark evidence | Not started | 20% | Depends on A3, A4, D2 |
| Operational confidence | Final safety/perf/readiness sweep | Partial | 35% | Needs end-to-end runtime maturity |

Release / quality-gate rollup: 58%

## Current Critical Sequence

1. Finish small review-driven runtime cleanup that directly supports A3.
2. Start A3: make graph/search traversal the primary scan execution path.
3. Keep the linear path as the explicit fallback shell during A3.
4. Run the A4 recall gate.
5. Only then proceed to planner activation, graph-aware insert, vacuum repair, and full SQL benchmark result generation.

## Current Major Blockers

| Blocker | Affects | Owner / lane |
| --- | --- | --- |
| Graph-first ordered scan runtime is not yet primary | `A3`, `A4`, `A5`, `A6`, `C1`, `D2` | Runtime lane |
| HNSW recall numbers are not yet measured on the real scan path | `A4`, `C1`, `D2` | Runtime lane |
| Graph-aware insert is not yet implemented | `A5`, `C1` drift benchmarks | Runtime lane |
| Vacuum graph repair is not yet implemented | `A6`, `C1` post-vacuum benchmarks | Runtime lane |
| ADR-011 planner gate is still active | `D2` | Planner lane after `A4` |
| SIMD and hot-path optimization would be premature before A3 stabilizes | `B1`, optimization work | Optimization lane |
