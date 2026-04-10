# Project Status

Last updated: 2026-04-09
Basis: post-A3 A4 debugging after landing the tiled-FWHT `1536` quantizer path and fixture-backed 10K gate helpers; synthetic/reference contradiction established; real-corpus recall lane added; SIMD branch at `d38e625`

## Reading Guide

- Percentages are judgment-based delivery estimates, not LOC metrics.
- `100%` means the intended v0.1 scope for that row is merged and validated on `main`.
- Infrastructure and harness completion does not count as benchmark, profiling, or optimization completion by itself.
- Rollups are weighted by delivery significance, not simple averages.

## Rollup

| Rollup | % Done | Meaning |
| --- | ---: | --- |
| Correctness-complete | 78% | Foundation/build solid; graph-first scan complete; graph-aware insert and vacuum repair remain |
| Test/validation-complete | 76% | Broad unit/integration/CI coverage exists, but graph-first scan validation and final unsafe hardening remain |
| Benchmark/profile-complete | 36% | Benchmark harnesses exist, but end-to-end HNSW latency, storage, and recall evidence is still mostly blocked |
| Optimization-complete | 18% | SIMD runtime dispatch and AVX2+NEON scoring landed on feature branch; merge and throughput proof pending A3 |
| Release-ready | 56% | Build packaging and quality infrastructure are in decent shape; cleanup sprint landed (sentinel fix, snapshot consolidation, dead code gating) |
| Total project completion | 72% | Weighted overall estimate to final intended scope |

## Execution Task Map

| ID | Task | Includes | Status | % Done | Notes |
| --- | --- | --- | --- | ---: | --- |
| `A1` | AM split | `scan`, `insert`, `build`, `options`, `cost`, `vacuum`, `routine`, `shared`, `search` module split | Done | 100% | Complete on `main` |
| `A2` | Graph/search traversal seam | Layer-0 traversal helpers, visible frontier protocol, bootstrap traversal boundary | Done | 100% | Landed as part of the A3 close arc |
| `A3` | Graph-first scan runtime | Make graph/search traversal the primary ordered scan path with linear fallback shell | **Done** | 100% | Cursor-owned graph-first runtime complete (reviews 182-193); bootstrap helpers gated to test/debug |
| `A4` | Recall gate | HNSW Recall@10 measurement and go/no-go threshold | **In progress — failing** | 70% | Repaired 10K harness still hard-fails (`8.4% / 21.8% / 26.8% / 35.3%`), but the production `1536` tiled-FWHT quantizer path now lifts exact-only `1k` Recall@10 to `77.0%` (uniform) / `81.5%` (clustered), a live `1k` graph-fixture probe passes `exact >= 70%`, `graph >= 70%`, and fixture-backed 10K gate helpers now separate reset from reusable reports; next bottleneck is still one-time 10K index build cost |
| `A5` | Graph-aware insert | Greedy descent, neighbor selection, backlinks, drift handling | Not started | 0% | Blocked on `A3`/`A4` |
| `A6` | Vacuum repair | Mark/repair/finalize vacuum with graph repair | Not started | 0% | Blocked on `A3`/`A4` |
| `B1` | SIMD | AVX2+FMA, NEON, runtime detection, equivalence tests, throughput proof | **In progress (coder-2)** | 25% | Runtime dispatch + AVX2/NEON scoring on feature branch `coder1-b1-simd-accel` |
| `B2` | CI / safety / quality | CI wiring, fuzz, miri, deny, layout checks, broader NFR-005 hardening | In progress | 80% | Cleanup sprint landed (sentinel fix, snapshot consolidation, dead code gating) |
| `C1` | Full benchmark suite | NFR-001/002/003 scripts, harnesses, reporting, end-to-end result artifacts | In progress | 45% | Infrastructure is built; final result runs are blocked on `A3`/`A5`/`A6` |
| `C2` | Real-corpus recall lane | External/real embedding corpus loader plus relation-backed A4 rerun on a spec-credible dataset | Ready | 10% | New Task 12; needed because raw reference HNSW is also weak on the current synthetic fixtures |
| `D1` | Planner scaffold | Cost-model scaffolding, explain/stat surfaces, PG18 read-stream scaffolding | **Done** | 90% | Merged to `main`; only PG18 callback bindings remain (need PG18 toolchain) |
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
| Bootstrap traversal seam | Graph/search ownership split, visible frontier protocol, graph-owned layer-0 traversal helpers | Done | 100% | Closed through the A3 cursor and frontier-ownership arc |
| Graph-first ordered execution | Make graph/search traversal primary in `amgettuple` | Done | 100% | Cursor-owned runtime complete; bootstrap helpers gated to test/debug |
| Linear fallback policy | Keep linear scan as explicit fallback shell during A3 | Done | 100% | Fallback is now explicit and only entered when graph traversal cannot produce an initial ordered result |
| `ef_search` runtime behavior | Resolved `ef_search` drives bootstrap frontier sizing | Mostly done | 85% | Main runtime wiring landed; sentinel cleanup remains elsewhere |
| Recall gate readiness | Runtime integrity sufficient to measure HNSW Recall@10 | Done | 100% | Repaired batched fixture harness plus fixture-backed gate helpers now support reusable 10K report surfaces, but one-time 10K reset/index build is still too expensive for an interactive loop |

Scan runtime rollup: 72%

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
| Planner scaffold | Cost/explain/stat/read-stream scaffolding | Done | 90% | Merged to `main`; only PG18 callback bindings remain |
| Planner activation | Real index selection and credible cost model | Not started | 5% | Gated on runtime/recall |
| PG18 async/read_stream integration | Runtime scan integration with PG18 path | Not started | 10% | Scaffold exists; production integration waits on scan |
| Strategy / EXPLAIN surfaces | FR-023 / FR-024 surfaces | Partial | 45% | Descriptive surfaces exist; activation still gated |

Planner / PG18 rollup: 42%

## 6. Testing / Validation

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Unit / property / layout tests | Scalar, page, codec, search protocol, size/layout checks | Strong | 92% | Broad low-level coverage exists |
| `cargo test` / `pgrx test` integration | Extension-level build and runtime integration | Strong | 82% | Good staged-behavior coverage exists |
| CI / safety tooling | Clippy, deny, fuzz, miri, benchmark-action, nightly checks | Strong | 75% | Base infrastructure is present |
| Graph-first runtime validation | Ordered scan behavior under A3 | In progress | 78% | Ordered-result regression is in place; repaired 10K evidence still fails, but the new live `1k` tiled-FWHT probe clears a `70%` graph Recall@10 floor at `(m=8, ef=128)` |
| Unsafe/stability audit | Final unsafe review and hardening pass | Partial | 50% | Tooling exists; final audit remains |

Testing / validation rollup: 76%

## 7. Benchmarking / Profiling

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Microbenchmark infrastructure | Criterion, iai-callgrind, dhat, Makefile targets, shared generators | Done | 100% | Harnesses are built and validated |
| Quantizer-level benchmark runs | Pure-Rust microbench and recall-smoke evidence | Strong | 80% | Useful baseline numbers exist |
| SQL benchmark infrastructure | `bench_sql_latency.sh`, `bench_storage.sh`, `bench_recall.py`, reporting template | Done | 90% | Scripts exist, but depend on working scan/insert/vacuum |
| End-to-end HNSW latency/storage results | NFR-001 and NFR-002 result artifacts | Not started | 0% | Blocked on A5/A6 and full benchmark runs |
| End-to-end HNSW recall results | NFR-003 result artifacts over built indexes | In progress — blocked on dataset lane | 50% | Repaired 10K synthetic graph-first run on 2026-04-08 still fails, the `1536` tiled-FWHT production path materially raises the cheap `1k` exact ceiling and clears a live `1k` graph probe, but new raw-reference baselines also show the current synthetic fixtures are not a credible gate surface; Task 12 / C2 now tracks the real-corpus rerun path |
| Runtime hot-path profiling | Real graph traversal profiling and bottleneck evidence | Not started | 10% | Premature before graph-first scan is primary |

Benchmarking / profiling rollup: 36%

## 8. Optimization / SIMD

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Scalar baseline | Working scalar quantizer and scan code paths | Partial | 55% | Correct baseline exists, but this is not the same as an optimization pass |
| Quantizer optimization passes | Deliberate score/encode/hadamard improvement work based on profiling | Not started | 10% | Benchmark harnesses exist, but no serious optimization campaign has been run yet |
| SIMD acceleration | AVX2+FMA, NEON, runtime detection, equivalence proof, throughput proof | **In progress** | 25% | Runtime dispatch + AVX2/NEON scoring paths on feature branch; equivalence tests and throughput proof pending |
| Runtime scan optimization | Tuning the graph-first scan hot path | Not started | 0% | Wait until A4 confirms scalar recall/correctness on the live graph path |
| Memory / buffer tuning | Traversal footprint, buffer behavior, allocator-pressure tuning | Not started | 5% | Some design notes exist, not a real tuning pass yet |

Optimization / SIMD rollup: 18%

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

1. **Coder-1:** A3 done — graph-first scan runtime is cursor-owned and live. **A4 is next: recall gate.**
2. **Coder-2:** B1 in progress — SIMD acceleration on feature branch. Merge after A4 confirms scalar correctness.
3. **Now:** A4 remains open, but the latest reference baselines show the current synthetic fixtures are not a credible gate surface by themselves: raw source-vector `hnsw-rs` reaches only `29.0%` (uniform) / `26.0%` (clustered) at `(m=8, ef=128)` and `66.5%` at `(m=16, ef=200)`. The highest-value next move is therefore C2 / Task 12: run the existing recall probes on a real `1536`-dimensional embedding corpus consistent with `NFR-003`.
4. After A4 is fixed and passes on a credible corpus: merge SIMD, D2 planner activation, A5 insert, A6 vacuum.
5. Full SQL benchmark result generation after A5/A6.

## Current Major Blockers

| Blocker | Affects | Owner / lane |
| --- | --- | --- |
| ~~Graph-first ordered scan runtime is not yet primary~~ | ~~`A3`, `A4`, `A5`, `A6`, `C1`, `D2`~~ | **Resolved** (A3 closed 2026-04-08) |
| Live graph-first recall gate still fails on the repaired 10K corpus (`21.8%` Recall@10 at `m=8, ef=128`), and raw reference HNSW is also weak on the current synthetic fixtures | `A4`, `C1`, `C2`, `D2` | Runtime / benchmark methodology lane |
| A4 still lacks a real `1536`-dimensional embedding corpus path consistent with `NFR-003` | `A4`, `C2`, `C1` | Benchmark methodology lane |
| Graph-aware insert is not yet implemented | `A5`, `C1` drift benchmarks | Runtime lane |
| Vacuum graph repair is not yet implemented | `A6`, `C1` post-vacuum benchmarks | Runtime lane |
| ADR-011 planner gate is still active | `D2` | Planner lane after `A4` |
| SIMD merge blocked on A3 scalar correctness confirmation | `B1` merge | Coder-2 (feature branch ready, merge after A3) |
