# Project Status

Last updated: 2026-04-11
Basis: A4 is closed on `main`, B1 SIMD is merged and validated on x86_64, A5 graph-aware insert is merged end-to-end on `main`, and A6 now lands vacuum mark/finalize behavior short of graph repair

## Reading Guide

- Percentages are judgment-based delivery estimates, not LOC metrics.
- `100%` means the intended v0.1 scope for that row is merged and validated on `main`.
- Infrastructure and harness completion does not count as benchmark, profiling, or optimization completion by itself.
- Rollups are weighted by delivery significance, not simple averages.

## Rollup

| Rollup | % Done | Meaning |
| --- | ---: | --- |
| Correctness-complete | 87% | Foundation/build solid; graph-first scan, recall gate, and graph-aware insert are complete; vacuum repair remains the main runtime gap |
| Test/validation-complete | 83% | Broad unit/integration/CI coverage exists, with real-corpus recall evidence, live-insert drift observability, and bounded stale-snapshot retry coverage now in hand; final unsafe hardening remains |
| Benchmark/profile-complete | 47% | Benchmark harnesses exist, the initial real-corpus recall gate is closed, and merged SIMD microbench evidence now exists on `main`; latency/storage and post-insert/vacuum suites remain |
| Optimization-complete | 42% | SIMD runtime dispatch, AVX2 FWHT/scoring, NEON 3-bit scoring, and prepared-query hot-path cuts are merged on `main`; scan-path tuning and aarch64 runtime validation remain |
| Release-ready | 66% | Build packaging and quality infrastructure are in decent shape, the initial recall gate is closed on real data, and the insert path is now graph-aware end-to-end |
| Total project completion | 81% | Weighted overall estimate to final intended scope |

## Execution Task Map

| ID | Task | Includes | Status | % Done | Notes |
| --- | --- | --- | --- | ---: | --- |
| `A1` | AM split | `scan`, `insert`, `build`, `options`, `cost`, `vacuum`, `routine`, `shared`, `search` module split | Done | 100% | Complete on `main` |
| `A2` | Graph/search traversal seam | Layer-0 traversal helpers, visible frontier protocol, bootstrap traversal boundary | Done | 100% | Landed as part of the A3 close arc |
| `A3` | Graph-first scan runtime | Make graph/search traversal the primary ordered scan path with linear fallback shell | **Done** | 100% | Cursor-owned graph-first runtime complete (reviews 182-193); bootstrap helpers gated to test/debug |
| `A4` | Recall gate | HNSW Recall@10 measurement and go/no-go threshold | **Done** | 100% | Closed on 2026-04-10: canonical real `10K` passes strongly (`97.1% / 97.3% / 97.4% / 97.5%`) and broader real `50K` gate evidence also passes (`50`-query gate: `92.6% / 94.4% / 94.8% / 95.2%`) |
| `A5` | Graph-aware insert | Greedy descent, neighbor selection, backlinks, drift handling | **Done** | 100% | Insert search, forward links, backlinks, overflow pruning, drift accounting, and bounded stale-snapshot retry hardening are merged on `main` |
| `A6` | Vacuum repair | Mark/repair/finalize vacuum with graph repair | In progress | 35% | Mark + finalize are merged on `main`; graph repair is the remaining major A6 step |
| `B1` | SIMD | AVX2+FMA, NEON, runtime detection, equivalence tests, throughput proof | **Substantially complete** | 90% | Merged on `main` on 2026-04-11; x86_64 validation and throughput proof are in hand, while aarch64 runtime validation still needs hardware |
| `B2` | CI / safety / quality | CI wiring, fuzz, miri, deny, layout checks, broader NFR-005 hardening | In progress | 80% | Cleanup sprint landed (sentinel fix, snapshot consolidation, dead code gating) |
| `C1` | Full benchmark suite | NFR-001/002/003 scripts, harnesses, reporting, end-to-end result artifacts | In progress | 45% | Infrastructure is built; final result runs are now mainly blocked on `A6` and the post-vacuum benchmark lane |
| `C2` | Real-corpus recall lane | External/real embedding corpus loader plus relation-backed A4 rerun on a spec-credible dataset | **Done for A4** | 100% | Loader, canonical subset contract, manifest verification, cheaper detached gate reruns, and the A4 signoff evidence on real `10K` / real `50K` are all landed on `main` |
| `D1` | Planner scaffold | Cost-model scaffolding, explain/stat surfaces, PG18 read-stream scaffolding | **Done** | 90% | Merged to `main`; only PG18 callback bindings remain (need PG18 toolchain) |
| `D2` | Planner activation | Real planner enablement, credible cost model, ADR-011 retirement, PG18 scan integration | Not started | 10% | No longer blocked on A4 recall evidence; still gated by ADR-011 retirement and downstream sequencing |

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
| Recall gate readiness | Runtime integrity sufficient to measure HNSW Recall@10 | Done | 100% | Repaired fixture and external real-corpus helpers now support reusable gate/report surfaces, including detached real-corpus gate capture on `main` |

Scan runtime rollup: 72%

## 3. Insert Path

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Current insert correctness | Shape validation, metadata setup, duplicate handling, tail-page append/reuse | Strong | 92% | Live insert shape, duplicate handling, append/reuse, metadata bookkeeping, and retry-aware backlink mutation are stable on `main` |
| Graph-aware insert | Greedy descent, neighbor selection, backlink repair | Done | 100% | Search, forward links, backlinks, overflow pruning, and bounded stale-snapshot retry hardening are merged |
| Insert drift accounting | Inserted-since-rebuild tracking and drift-aware measurement | Done | 100% | `tqhnsw_index_admin_snapshot(regclass)` now exposes live-node count, inserted-since-rebuild, and drift fraction |
| Insert decontention follow-up | Metadata, tail-page, and backlink hotspot reduction | Planned | 10% | Explicitly tracked in `plan/tasks/13-insert-throughput.md`; ADR-026 ordering remains the safety baseline |

Insert path rollup: 88%

## 4. Vacuum / Repair

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Vacuum callback scaffold | Vacuum module split and base callback structure | Strong | 82% | Callback-driven mark/finalize behavior is now live |
| Graph repair logic | Mark, repair, finalize passes over deleted nodes | Partial | 35% | Mark + finalize landed; graph repair still remains |
| Stats cleanup integration | `amvacuumcleanup` and stats alignment | Partial | 50% | Live-element counts now flow through cleanup; pg_class-facing proof still remains |

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
| Graph-first runtime validation | Ordered scan behavior under A3 | Strong | 92% | Ordered-result regression is in place, canonical real `10K` passes strongly, and broader real `50K` slices stay comfortably above the A4 gate |
| Unsafe/stability audit | Final unsafe review and hardening pass | Partial | 50% | Tooling exists; final audit remains |

Testing / validation rollup: 76%

## 7. Benchmarking / Profiling

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Microbenchmark infrastructure | Criterion, iai-callgrind, dhat, Makefile targets, shared generators | Done | 100% | Harnesses are built and validated |
| Quantizer-level benchmark runs | Pure-Rust microbench and recall-smoke evidence | Strong | 84% | Useful baseline numbers exist, and the merged SIMD lane now has current-main vs merged Criterion evidence for `prepare_ip_query` and `score_ip_encoded` |
| SQL benchmark infrastructure | `bench_sql_latency.sh`, `bench_storage.sh`, `bench_recall.py`, reporting template | Done | 90% | Scripts exist, but depend on working scan/insert/vacuum |
| End-to-end HNSW latency/storage results | NFR-001 and NFR-002 result artifacts | Not started | 0% | Blocked on A6 and full benchmark runs |
| End-to-end HNSW recall results | NFR-003 result artifacts over built indexes | Strong | 78% | The initial real-corpus signoff surface is now closed: canonical real `10K` passes strongly and the broader real `50K` `50`-query gate reports `92.6% / 94.4% / 94.8% / 95.2%`; broader post-gate reporting remains under `C1` |
| Runtime hot-path profiling | Real graph traversal profiling and bottleneck evidence | Not started | 10% | Premature before graph-first scan is primary |

Benchmarking / profiling rollup: 42%

## 8. Optimization / SIMD

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Scalar baseline | Working scalar quantizer and scan code paths | Done | 100% | Scalar reference paths are still present and remain the comparison baseline for SIMD validation |
| Quantizer optimization passes | Deliberate score/encode/hadamard improvement work based on profiling | Strong | 72% | The merged B1 lane now includes padded-SRHT query prep, prepared-query LUT cuts, and AVX2 FWHT/scoring improvements with current-main benchmark evidence |
| SIMD acceleration | AVX2+FMA, NEON, runtime detection, equivalence proof, throughput proof | **Mostly done** | 90% | Merged on `main`; x86_64 equivalence + throughput proof are complete, and NEON implementation is present but still needs aarch64 runtime validation |
| Runtime scan optimization | Tuning the graph-first scan hot path | Not started | 10% | A4 is closed; further tuning is now post-gate optimization work rather than recall rescue |
| Memory / buffer tuning | Traversal footprint, buffer behavior, allocator-pressure tuning | Not started | 5% | Some design notes exist, not a real tuning pass yet |

Optimization / SIMD rollup: 18%

## 9. Docs / Specs / Coordination

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Spec / ADR corpus | FR/NFR/ADR coverage and decision boundaries | Strong | 85% | Good structure already exists |
| Task planning surface | Project plan and task docs | Improving | 78% | Mostly solid; still being tightened for scanability |
| Review memory | Review request and feedback trail | Strong | 92% | Detailed history exists in `review/*` |
| Status reporting | Stable project-level status snapshot | In progress | 68% | This file is intended to make status easier to read |

Docs / coordination rollup: 82%

## 10. Release / CI / Quality Gates

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Extension packaging/build | Extension packaging and local build/install path | Done | 100% | Solid |
| Quality gate infrastructure | CI, deny, miri, fuzz, benchmark-action, validation commands | Strong | 75% | Good base exists |
| Release proof | Recall signoff, planner/runtime readiness, benchmark evidence | Partial | 40% | Recall signoff is now in hand; planner/runtime maturity and broader benchmark evidence remain |
| Operational confidence | Final safety/perf/readiness sweep | Partial | 35% | Needs end-to-end runtime maturity |

Release / quality-gate rollup: 62%

## Current Critical Sequence

1. **Coder-1:** A4 is closed — graph-first scan recall now has real-corpus signoff evidence on `main`.
2. **Next runtime lane:** Continue A6 from the landed mark/finalize slice into graph repair.
3. **Coder-2 follow-up:** B1 SIMD is merged on `main`; only aarch64 runtime validation remains, and it is no longer on the critical path.
4. **Planner:** D2 is no longer blocked on A4 recall evidence, but ADR-011 retirement and planner sequencing still remain.
5. Full SQL benchmark result generation after A6, with insert decontention tracked separately in Task 13.

## Current Major Blockers

| Blocker | Affects | Owner / lane |
| --- | --- | --- |
| ~~Graph-first ordered scan runtime is not yet primary~~ | ~~`A3`, `A4`, `A5`, `A6`, `C1`, `D2`~~ | **Resolved** (A3 closed 2026-04-08) |
| Synthetic `10K` still fails badly and remains misleading as a benchmark surface | `C1`, post-gate methodology work | Benchmark methodology lane |
| Vacuum graph repair is not yet implemented | `A6`, `C1` post-vacuum benchmarks | Runtime lane |
| ADR-011 planner gate is still active | `D2` | Planner lane after A4 |
| aarch64 SIMD runtime validation still needs hardware | `B1` | Coder-2 / validation lane |
