# Project Status

Last updated: 2026-05-06
Basis: A4 is closed on `main` using canonical DBpedia-derived real-corpus evidence, B1 SIMD is merged and validated on x86_64, A5 graph-aware insert is merged end-to-end on `main`, A6 vacuum repair is complete on `main`, C1 now has durable real-corpus NFR-001 latency artifacts plus a verified warm-cache seam on `main`, PG18 shared infrastructure is live with PG17 fallback preserved, Task 26 PG18 HNSW concurrent-DSM parallel build has completed its local scale lane through the 990k DBPedia anchor, Task 28 has completed the first local IVF tuning slice on the `task28-ivf` branch, and Task 30 SPIRE IVF is active on the Phase 4 local multi-store placement branch.

## Reading Guide

- Percentages are judgment-based delivery estimates, not LOC metrics.
- `100%` means the intended v0.1 scope for that row is merged and validated on `main`.
- Infrastructure and harness completion does not count as benchmark, profiling, or optimization completion by itself.
- Rollups are weighted by delivery significance, not simple averages.

## Rollup

| Rollup | % Done | Meaning |
| --- | ---: | --- |
| Correctness-complete | 91% | Foundation/build solid; graph-first scan, recall gate, graph-aware insert, and vacuum repair are complete; planner activation and PG18 runtime work remain |
| Test/validation-complete | 87% | Broad unit/integration/CI coverage exists, with real-corpus recall evidence, live-insert drift observability, bounded stale-snapshot retry coverage, and a 60-second vacuum concurrency harness now in hand; final unsafe hardening remains |
| Benchmark/profile-complete | 58% | Benchmark harnesses exist, the initial real-corpus recall gate is closed, and durable real-corpus latency artifacts now exist on `main`; storage and broader post-insert/vacuum suites still remain |
| Optimization-complete | 49% | SIMD runtime dispatch, AVX2 FWHT/scoring, NEON 3-bit scoring, prepared-query hot-path cuts, and the first real scan-path wins are merged on `main`; deeper scan tuning and aarch64 runtime validation remain |
| Release-ready | 70% | Build packaging and quality infrastructure are in decent shape, the initial recall gate is closed on real data, and vacuum repair now has concurrency validation on `main` |
| Total project completion | 84% | Weighted overall estimate to final intended scope |

## Execution Task Map

| ID | Task | Includes | Status | % Done | Notes |
| --- | --- | --- | --- | ---: | --- |
| `A1` | AM split | `scan`, `insert`, `build`, `options`, `cost`, `vacuum`, `routine`, `shared`, `search` module split | Done | 100% | Complete on `main` |
| `A2` | Graph/search traversal seam | Layer-0 traversal helpers, visible frontier protocol, bootstrap traversal boundary | Done | 100% | Landed as part of the A3 close arc |
| `A3` | Graph-first scan runtime | Make graph/search traversal the primary ordered scan path with linear fallback shell | **Done** | 100% | Cursor-owned graph-first runtime complete (reviews 182-193); bootstrap helpers gated to test/debug |
| `A4` | Recall gate | HNSW Recall@10 measurement and go/no-go threshold | **Done** | 100% | Closed on 2026-04-10: canonical real `10K` passes strongly (`97.1% / 97.3% / 97.4% / 97.5%`) and broader real `50K` gate evidence also passes (`50`-query gate: `92.6% / 94.4% / 94.8% / 95.2%`) |
| `A5` | Graph-aware insert | Greedy descent, neighbor selection, backlinks, drift handling | **Done** | 100% | Insert search, forward links, backlinks, overflow pruning, drift accounting, and bounded stale-snapshot retry hardening are merged on `main` |
| `A6` | Vacuum repair | Mark/repair/finalize vacuum with graph repair | **Done** | 100% | Mark, repair, finalize, and the 60-second INSERT + scan + VACUUM concurrency harness are merged on `main` |
| `B1` | SIMD | AVX2+FMA, NEON, runtime detection, equivalence tests, throughput proof | **Substantially complete** | 90% | Merged on `main` on 2026-04-11; x86_64 validation and throughput proof are in hand, while aarch64 runtime validation still needs hardware |
| `B2` | CI / safety / quality | CI wiring, fuzz, miri, deny, layout checks, broader NFR-005 hardening | In progress | 80% | Cleanup sprint landed (sentinel fix, snapshot consolidation, dead code gating) |
| `C1` | Full benchmark suite | NFR-001/002/003 scripts, harnesses, reporting, end-to-end result artifacts | In progress | 66% | Durable real-corpus NFR-001 artifacts now exist on `main`, and the launcher now supports verified warm per-cell runs, but the honest warm `10K` surface is still about `p50=14.3ms` at `m=8, ef_search=40` and NFR-002 remains open |
| `T26` | PG18 HNSW parallel build | Concurrent DSM graph assembly, worker-headroom validation, 50k/990k scale packets | **Done for local scale lane** | 100% | `task28-ivf` carries the default-on PG18 concurrent DSM path. Packet 672 recorded real 50k improving from `07:12.017` at 1 worker to `02:27.948` at 8 launched graph workers after worker headroom was fixed; packet 669 proved the 990k DBPedia worker launch path but still took `01:31:57.326`, motivating the IVF pivot. |
| `T28` | IVF initial tuning | `ec_ivf` profile, IVF heap-f32 rerank, rerank width, local DBPedia 10k/25k grids | **Checkpoint complete** | 90% | Local PG18 tuning found a usable first frontier: packet 30043 has 10k `nlists=32,nprobe=24,width=25` at `0.9980` recall@10 and about `135/146 ms` p50/p95; packet 30044 has 25k `32/32,width=25` at `1.0000` and about `435/456 ms`. Product claims still require a later Graviton-class benchmark. |
| `T30` | SPIRE IVF foundation | Partition-object storage, recursive routing, update mechanics, Phase 4 local multi-store placement | **In progress** | 90% | Phase 1 measured recall/latency is recorded in packet 30530, mutation-path local-store routing is in packet 30531, scan prefetch placement resolution is in packet 30532, local placement benchmark evidence is in packet 30533, PG18 ReadStream-backed local fetch is in packet 30534, and SQL VACUUM multi-store coverage now proves post-build insert/delete/vacuum retains both local store relations while ordered scan hides the deleted row and returns the inserted row. The local benchmark covers one-store, same-device two-store, and `/mnt/e` two-store lanes while reserving production claims for future cloud hardware. Remaining Task 30 gates are PQ-FastScan scorer binding and physical object reclamation/old-epoch cleanup. |
| `C2` | Real-corpus recall lane | External/real embedding corpus loader plus relation-backed A4 rerun on a spec-credible dataset | **Done for A4** | 100% | Loader, canonical subset contract, manifest verification, cheaper detached gate reruns, and the A4 signoff evidence on real `10K` / real `50K` are all landed on `main` |
| `D1` | Planner scaffold | Cost-model scaffolding, explain/stat surfaces, PG18 read-stream scaffolding | **Done** | 100% | The scaffolded seams are complete, and their planned PG18 bindings are now live on `pg18-shared-infra-merge` |
| `D2` | Planner activation | Real planner enablement, credible cost model, ADR-011 retirement, PG18 scan integration | **In review** | 95% | Cost model, ADR-011 retirement, PG18 callbacks, EXPLAIN hooks, ReadStream wiring, and module identity are live on `pg18-shared-infra-merge`; the remaining gate is preload-time shared pgstat activation coverage plus post-merge follow-through |

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
| `ef_search` runtime behavior | Resolved `ef_search` drives bootstrap frontier sizing | Done | 100% | Sentinel cleanup landed in commit `bb13a7a` (`TQHNSW_SESSION_EF_SEARCH_UNSET = -1`); runtime, GUC, and snapshot helpers all consume the resolved value |
| Recall gate readiness | Runtime integrity sufficient to measure HNSW Recall@10 | Done | 100% | Repaired fixture and external real-corpus helpers now support reusable gate/report surfaces, including detached real-corpus gate capture on `main` |

Scan runtime rollup: 72%

## 3. Insert Path

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Current insert correctness | Shape validation, metadata setup, duplicate handling, tail-page append/reuse | Strong | 92% | Live insert shape, duplicate handling, append/reuse, metadata bookkeeping, and retry-aware backlink mutation are stable on `main` |
| Graph-aware insert | Greedy descent, neighbor selection, backlink repair | Done | 100% | Search, forward links, backlinks, overflow pruning, and bounded stale-snapshot retry hardening are merged |
| Insert drift accounting | Inserted-since-rebuild tracking and drift-aware measurement | Done | 100% | `ec_hnsw_index_admin_snapshot(regclass)` now exposes live-node count, inserted-since-rebuild, and drift fraction |
| Insert decontention follow-up | Metadata, tail-page, and backlink hotspot reduction | Planned | 10% | Explicitly tracked in `plan/tasks/13-insert-throughput.md`; ADR-026 ordering remains the safety baseline |

Insert path rollup: 88%

## 4. Vacuum / Repair

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Vacuum callback scaffold | Vacuum module split and base callback structure | Done | 100% | Callback-driven mark/repair/finalize behavior is live and concurrency-validated |
| Graph repair logic | Mark, repair, finalize passes over deleted nodes | Done | 100% | Mark, dead-edge unlink, layer-aware replacement fill, finalize, and the concurrent safety proof are merged |
| Stats cleanup integration | `amvacuumcleanup` and stats alignment | Partial | 55% | Live-element counts now flow through cleanup; pg_class-facing proof still remains |

Vacuum / repair rollup: 72%

## 5. Planner / PG18

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Planner scaffold | Cost/explain/stat/read-stream scaffolding | Done | 100% | The original scaffold is complete and the shared PG18 bindings are now live on `pg18-shared-infra-merge` |
| Planner activation | Real index selection and credible cost model | **In review** | 95% | D2 now has live costing, `amgettreeheight`, strategy translation, and planner/diagnostics snapshots; the remaining PG18 blocker is preload-time shared pgstat activation rather than callback/toolchain bring-up |
| PG18 async/read_stream integration | Runtime scan integration with PG18 path | Strong | 90% | Graph-neighbor prefetch, linear fallback reads, and vacuum tuple counting now use PG18 ReadStream wiring on `pg18-shared-infra-merge`; follow-on work is measurement, not callback registration |
| Strategy / EXPLAIN surfaces | FR-023 / FR-024 surfaces | Strong | 90% | Strategy translation and EXPLAIN option/per-node hook wiring are live on `pg18-shared-infra-merge`; shared pgstat still depends on preload-time activation |

Planner / PG18 rollup: 78%

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
| SQL benchmark infrastructure | `ecaz bench latency`, `ecaz bench storage`, `ecaz bench recall`, reporting template | Done | 90% | CLI surfaces exist, but depend on working scan/insert/vacuum |
| End-to-end HNSW latency/storage results | NFR-001 and NFR-002 result artifacts | In progress | 42% | Durable real-corpus NFR-001 latency artifacts now exist on `main` (`m=8` canonical, `m=16` isolated), but they still miss the latency target badly and NFR-002 storage artifacts are still pending |
| End-to-end HNSW recall results | NFR-003 result artifacts over built indexes | Strong | 78% | The initial real-corpus signoff surface is now closed: canonical real `10K` passes strongly and the broader real `50K` `50`-query gate reports `92.6% / 94.4% / 94.8% / 95.2%`; broader post-gate reporting remains under `C1` |
| Runtime hot-path profiling | Real graph traversal profiling and bottleneck evidence | Strong | 68% | Real `10K` hot-path profiling is now in hand; graph rereads and then repeated scoring were both measured and converted into merged scan-path wins |

Benchmarking / profiling rollup: 42%

## 8. Optimization / SIMD

| Area | Includes | Status | % Done | Notes |
| --- | --- | --- | ---: | --- |
| Scalar baseline | Working scalar quantizer and scan code paths | Done | 100% | Scalar reference paths are still present and remain the comparison baseline for SIMD validation |
| Quantizer optimization passes | Deliberate score/encode/hadamard improvement work based on profiling | Strong | 72% | The merged B1 lane now includes padded-SRHT query prep, prepared-query LUT cuts, and AVX2 FWHT/scoring improvements with current-main benchmark evidence |
| SIMD acceleration | AVX2+FMA, NEON, runtime detection, equivalence proof, throughput proof | **Mostly done** | 90% | Merged on `main`; x86_64 equivalence + throughput proof are complete, and NEON implementation is present but still needs aarch64 runtime validation |
| Runtime scan optimization | Tuning the graph-first scan hot path | In progress | 44% | Scan-local graph-read and score caches are merged with real-corpus latency wins, and warm per-cell measurement is now separated from per-query backend churn, but the current NFR-001 surface is still above target |
| Memory / buffer tuning | Traversal footprint, buffer behavior, allocator-pressure tuning | In progress | 22% | Shared-buffer churn has already been cut materially on the real `10K` path, but more runtime/memory tuning is still available |

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
2. **Task 26 / Task 28 branch:** PG18 HNSW concurrent DSM parallel build has a completed local scale lane, and IVF initial tuning now has a measured local frontier plus heap-f32 rerank correctness fixes. Landing should split Task 26 first, then stack the IVF PR.
3. **Coder-2 follow-up:** B1 SIMD is merged on `main`; only aarch64 runtime validation remains, and it is no longer on the critical path.
4. **Planner:** `main` now has live PG18 callback bindings, EXPLAIN hooks,
   ReadStream scan/vacuum wiring, shared pgstat registration via the preload-aware shim, and
   module identity with PG17 fallback preserved. The preload-aware activation lane now exists in
   repo, so the remaining follow-ons are measurement and optional parallel-scan work rather than
   PG18 toolchain bring-up.
5. Full SQL benchmark result generation after A6, with insert decontention tracked separately in Task 13. IVF product claims remain blocked on a dedicated Graviton-class benchmark; DiskANN remains task 29.

## Current Major Blockers

| Blocker | Affects | Owner / lane |
| --- | --- | --- |
| ~~Graph-first ordered scan runtime is not yet primary~~ | ~~`A3`, `A4`, `A5`, `A6`, `C1`, `D2`~~ | **Resolved** (A3 closed 2026-04-08) |
| Synthetic `10K` still fails badly and remains misleading as a benchmark surface | `C1`, post-gate methodology work | Benchmark methodology lane |
| Durable NFR-001 artifacts now exist, and verified warm per-cell runs reduce the real `10K` baseline materially, but both surfaces still miss spec (`cold` canonical `m=8, ef_search=40`: `p50=50.283ms`, `p99=55.862ms`; `warm` per-cell after 3 prime passes: `p50=14.315ms`, `p99=17.613ms` vs `<5ms` / `<15ms`) | `C1` | Runtime optimization lane |
| ~~ADR-011 planner gate is still active~~ | ~~`D2`~~ | **Resolved** (D2 cost-model activation, 2026-04-11; ADR-011 marked SUPERSEDED) |
| aarch64 SIMD runtime validation still needs hardware | `B1` | Coder-2 / validation lane |
| Task 26 / Task 28 branch still needs split/rebase/PR mechanics | Landing on `main` | Runtime-index lane |
