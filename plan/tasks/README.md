# Task Breakdown

These task files are the parallel execution breakdown for `tqvector`.

## Completed

1. `01-quantizer-core.md` — Phase 1
2. `02-datum-and-io.md` — Phase 2 (type/I/O)
3. `03-sql-surface.md` — Phase 2 (functions/operators)
4. `04-page-layout-and-wal.md` — Phase 3

## Archived Legacy Snapshots

These pre-lane task files are retained only for historical context under
`plan/tasks/archive/`. They are not live task numbers:

- `archive/05-build-and-scan.md`
- `archive/06-vacuum-and-insert.md`
- `archive/07-simd-and-benchmarks.md`
- `archive/08-safety-and-ci.md`

## Active Tracks

### Agent 1: Runtime / Index Core (critical path)

5. `05-graph-scan.md` — A1-A4 (**done on `main`**)
6. `06-graph-insert.md` — A5 (**done on `main`**)
7. `07-vacuum.md` — A6 (**complete on `main`**)

### Agent 2: Planner Integration

11. `11-planner.md` — D1/D2 substantially complete on `main`; remaining follow-on is measurement, with parallel-scan callbacks shelved

### Agent 3: SIMD / CI

8. `08-simd.md` — B1 (**can start now**)
9. `09-ci-and-safety.md` — B2 (mostly complete)
12. `12-real-corpus-recall.md` — C2 (**can start now**; resolves the A4 / NFR-003 real-dataset lane)

### Post-Gate

10. `10-benchmarks.md` — C1 (infrastructure complete, NFR runs blocked on 05)
13. `13-insert-throughput.md` — post-A5 decontention follow-up for metadata/tail-page/backlink hotspots
14. `14-adr030-v2-grouped-index.md` — long-horizon index-v2 grouped search-code redesign (feasibility + metadata contract)
15. `15-pqfastscan-first-class.md` — executes ADR-032: rename ScalarV1→TurboQuant / GroupedV2→PqFastScan, reloption selector, insert+vacuum parity; blocks merge to `main`
16. `16-turboquant-iteration.md` — post-task-15 follow-up: port binary prefilter, heap-f32 rerank, and hot/cold payload split from PqFastScan onto TurboQuant
17. `17-diskann-access-method.md` — second access method for 500M–3B-scale disk-resident indexes (owned by a separate agent; ADR-034)
18. `18-parallel-index-scan.md` — shelved indefinitely; not the current scaling-research frontier
19. `19-pg18-completion.md` — substantially complete on `main`: PG18 primary-target, amgettreeheight / EXPLAIN hook / pgstat-kind / ReadStream live, PG17 fallback preserved; ReadStream measurement remains
20. `20-opq-rotation.md` — executes ADR-036: OPQ as alternative transform front-end for PqFastScan, +10–20% recall per byte, zero scan-kernel change
21. `21-simd-modernization.md` — executes ADR-039 + task-08 hot-path follow-up: AVX-512 specializations and ARM SVE/SVE2 backend under existing runtime dispatch
22. `22-additive-residual-quantization.md` — executes ADR-037: **evaluate-gated** feasibility study of AQ / RVQ as PqFastScan successor; three decision gates, shelf-on-fail OK
23. `23-lsq-codebook-refinement.md` — executes ADR-038: drop-in k-means replacement, +2–5% recall, no wire format change, low priority fill-in
24. `24-post-native-build-storage-and-lever4-followons.md` — post-ADR-042 follow-up: reopen ADR-044 on a stable native builder, carry forward the `EXTENDED` / `MAIN` build-collapse bug, and close the lever-4 `ef_search` matrix before any persisted-default decision
25. `25-rabitq-quantizer.md` — RaBitQ quantizer landed on `main` and is wired into IVF via `storage_format = 'rabitq'` / `quantizer = 'rabitq'`; Symphony is no longer the active consumer.
26. `26-parallel-index-build.md` — HNSW parallel build landed for eligible PG18 builds; larger scale curves are deferred to AWS/RDS-class benchmark hardware
27. `27-symphony-access-method.md` — shelved indefinitely; the historical Symphony plan remains for reference only and requires a new accepted ADR to reopen.
28. `28-ivf-access-method.md` / `28-ivf-competitive-substrate.md` — IVF access method and local competitive substrate landed on `main`; larger product benchmarks remain deferred to dedicated hardware.
29. `29-diskann-initial-tuning.md` — DiskANN Task 29/29a/29b/29c/29d landed on `main`; `29e-diskann-rerank-cleanup-evidence.md` records follow-up cleanup/evidence and is not a current blocker.
30. `30-spire-ivf-foundation.md` — proposed ADR-049 implementation plan: reconcile landed IVF with SPIRE-compatible partition-object storage, build a single-level foundation, then add recursion, local multi-NVMe placement, boundary replication, top-level graph routing, and multi-machine placement.
    - `task30-phase9-spire-graph-architecture.md` — expanded SPIRE graph architecture track: top-graph frontier, scalable graph storage, global recursive beam, boundary replication contract, and global vector identity.
    - `task30-phase10-spire-execution-performance.md` — expanded SPIRE execution/performance track: bounded candidate collection, AM scan shape, heap rerank I/O, multi-NVMe read overlap, remote libpq executor, and performance harness.
    - `task30-phase11-spire-distributed-production-parity.md` — functional CustomScan/ADR-069 distributed read/write delivery: `EcSpireDistributedScan`, tuple payloads, placement directory, coordinator-routed INSERT/UPDATE/DELETE/PK SELECT, Stage E matrix evidence, and materialization-path cleanup.
    - `task30-phase12-spire-production-hardening.md` — production hardening before AWS: non-happy-path fixtures, typed tuple transport, planner/cache/cost hardening, 2PC recovery/cancel/concurrency, schema/type/isolation coverage, local multi-instance and multi-store readiness, and operator runbooks.
    - `task30-phase13-spire-aws-verification.md` — entry/exit gate for the final AWS-cloud verification phase after Phase 12 (RDS / Aurora rejected because they cannot load the ecaz custom AM / CustomScan; baseline is self-managed PG18 on EC2). Decomposed into:
        - `task30-phase13a-spire-aws-verification-design.md` — topology, datasets, workload matrix, pass/fail thresholds, observability surface, fault drills, packet skeleton, operator surface, cost guardrails, open reviewer decisions.
        - `task30-phase13b-spire-aws-verification-runbook.md` — operator runbook backed by the `infra/spire-aws/` Terraform module and `scripts/spire-aws/` orchestration scripts; one Makefile target per stage and one-shot `pass-correctness` / `pass-representative` passes.
        - `task30-phase13c-spire-aws-readiness-followups.md` — final local AWS-readiness blocker fixes for remote libpq TLS handling and PK SELECT schema-drift enforcement.
        - `task30-phase13d-spire-read-efficiency-observability.md` — final production-read measurement and low-risk efficiency fixes before AWS read workload execution: live profile rows, candidate/heap session reuse, cheap default diagnostics, and bounded merge work.
        - `task30-phase13e-spire-aws-production-gap-closure.md` — P0 gap-closure implementation track for static remote placement, production CustomScan reads, fanout evidence, and evidence-gated pooling before AWS product-scale proof.
31. `31-ivf-m5-optimization.md` — first-priority M5 optimization lane for landed IVF: refresh local baselines, classify the dominant bottleneck, and land one measured scan/scoring/churn optimization at a time.
32. `32-diskann-m5-optimization.md` — second-priority M5 optimization lane for landed DiskANN: refresh Task 29d baselines, profile low-L scan latency, and pursue targeted constant-factor wins without lowering recall floors.
33. `33-hnsw-m5-optimization.md` — third-priority M5 optimization lane for HNSW: refresh reference worker curves, then decide between direct DSM ingestion, offline/staged build, or narrow scan/build hot-path work.
34. `34-comprehensive-hardening.md` — local-first expansion of the ECAZ/SPIRE hardening stack: cargo-audit/deny/vet, Miri, cargo-careful, sanitizers, fuzzing, SQLsmith, Loom/Shuttle, Kani/Flux/MIRAI, Rudra, and unsafe-surface reporting.
35. `35-unsafe-quality-burndown.md` — **complete** (2026-05-19, closeout `5bc35c9a`): reviewed burndown of the grandfathered unsafe-comment baseline to zero; ~3,397 entries cleared across ~120 packets, four AM closeouts on file (083 SPIRE, 104 HNSW, 107 DiskANN, 122 IVF) plus top-level closeout 121. Structural follow-on tracked by Task 50.
36. `36-simd-scalar-differential.md` — proptest harness comparing every SIMD scoring / decoding path to a scalar reference; closes the silent-recall-regression gap left by Task 34's Miri scalar fallback.
37. `37-crash-recovery-and-amcheck.md` — PG18 crash-recovery harness with `SIGKILL` at WAL boundaries and `pg_amcheck` post-restart verification for every ECAZ AM; the WAL-replay safety net Task 34 explicitly does not cover.
38. `38-pg-fault-injection.md` — I/O (EIO/ENOSPC), palloc OOM, `pg_cancel_backend`, `statement_timeout`, and resource-exhaustion sweeps against ECAZ entry points, with buffer-pin / LWLock leak detection.
39. `39-test-quality-measurement.md` — `cargo-llvm-cov` coverage gate and `cargo-mutants` mutation testing over critical correctness modules; answers "are our tests real" for the existing and new hardening lanes.
40. `40-concurrency-model-checking-real.md` — retargets the Task 34 placeholder Loom/Shuttle harnesses at real ECAZ state machines (parallel build slots, SPIRE coordinator) via the lifted-module pattern, plus madsim / turmoil for SPIRE remote.
41. `41-ffi-safety-boundary.md` — inventory and enforcement for panic-across-FFI, `pg_guard`, palloc memory-context lifetimes, and RAII wrappers for PG buffer pins / LWLocks / snapshots; backed by a custom `dylint` lint suite.
42. `42-on-disk-format-invariants.md` — endian-explicit encoding, `qemu` cross-arch decode lane, static `size_of` / offset assertions for every on-disk type, a `(format_version, AM)` upgrade matrix, and `pg_upgrade` smoke.
43. `43-miri-careful-depth.md` — Tree Borrows pass, `-Zmiri-many-seeds` interleavings, and Miri/cargo-careful coverage extended to SPIRE coordinator, DiskANN/HNSW graph helpers, top-k merge, remote parser, and serialization.
43b. `43b-miri-careful-exhaustive-safety.md` — depth extension to Task 43 once Task 35 lands: Nth concurrent primitives under many-seeds, SPIRE careful micro-harness via extraction, proptest/cargo-fuzz adversarial lanes, per-test mutation probes, and `unsafe`-block coverage audit. Parked until Task 35 unsafe burndown completes; re-scoped against post-35 surface before execution.
44. `44-formal-verification-expansion.md` — Kani proofs for tuple alignment, payload length, leaf V2 metadata, top-k merge order, partition routing, and remote-parser rejection; Flux refinement types on real quantizer / page APIs (replaces the Task 34 synthetic Flux harness).
45. `45-static-analysis-and-supply-chain-depth.md` — custom `dylint` lints, `cargo-public-api` and `cargo-semver-checks` for API/ABI stability, SBOM generation, `cargo-vet` criteria delegation, license allow-listing, reproducible build checks, and yank watch.
46. `46-structure-aware-and-grammar-fuzzing.md` — `arbitrary`-derived structure-aware libFuzzer targets, ECAZ-grammar SQLsmith biased toward vector-operator / CustomScan paths, Honggfuzz + AFL+ cross-pollination, and corpus minimization.
47. `47-recall-and-cost-model-gates.md` — brute-force exact-KNN differential per AM with documented recall floors, cross-AM consistency (Jaccard / Kendall-tau), and an `EXPLAIN`-diff cost-model regression gate.
48. `48-build-matrix-and-soak.md` — CI matrix for darwin / linux-gnu / linux-musl × pg17 / pg18, qemu cross-endian decode lane, 24-hour soak harness with leak-slope detection, and PG resource-limit exhaustion sweeps.
49. `49-hardening-ci-governance.md` — recommended next coder pickup; retargets the four Task 34 synthetic harnesses (Rudra/Flux/Loom/Shuttle) at real ECAZ code, restores `make test` to `cargo test` on CI, documents the local → PR → nightly → weekly promotion ladder, and adds `make hardening-validate` to block future synthetic-only lanes.
50. `50-unsafe-structural-reduction.md` — post-Task-35 follow-on: reduce the *count* of `unsafe { ... }` blocks (not baseline entries) in the densest residual modules via encapsulation, type-lifted invariants, narrowed blocks, container-owned state, and closure APIs; gated on per-packet bench evidence (HNSW/IVF/DiskANN/SPIRE) showing no regression.
51. `51-ivf-rabitq-second-optimization-round.md` — follow-on AWS RaBitQ/IVF latency lane: paired same-host comparator baseline, 1M scan counters, `nlists`/nprobe geometry sweeps, local batch-decode scratch-SoA prototype, heap-rerank locality, adaptive nprobe/rerank width, and a gated Posting Layout v2 decision before any invasive on-disk format work.
59. `59-diskann-aws-graviton-tuning-1m-suite.md` — AWS Graviton DiskANN tuning lane after the Task 55 scan-materialization win: profile the remaining relation-read/decode/allocation/scoring and host-shape costs, tune the Graviton profile/config, land targeted optimizations, and prove with a full `ecaz bench suite` through 1M rows; Intel is deferred and active AWS profiles stay up unless the operator requests teardown.
60. `60-ec-diskann-rabitq-storage-format.md` — add `storage_format = 'rabitq'` to `ec_diskann` so operators can trade DiskANN's graph-traversal recall/latency curve against RaBitQ's much smaller on-disk footprint; target ≥30% index size reduction vs `pq_fastscan` at 1M with measured recall delta and packet 005's materialization-avoidance win preserved.
61. `61-hnsw-aws-graviton-first-pass.md` — AWS Graviton first-pass tuning lane for `ec_hnsw`: establish repeatable low-cost Graviton baselines at 10k/50k/100k and 1M if feasible, identify whether build, scan, memory, or storage dominates, land only evidence-backed narrow tuning, and keep Intel comparator work deferred.
62. `62-hnsw-graviton-full-optimization.md` — follow-through HNSW Graviton lane after Task 61: benchmark TurboQuant and PqFastScan on isolated 10k/50k/100k HNSW surfaces, decide whether the next wins are general HNSW, PqFastScan-specific, or Graviton-specific, and land only packet-backed optimization slices.
63. `63-hnsw-rabitq-storage-format.md` — design-gated storage-format task for adding `storage_format = 'rabitq'` to `ec_hnsw`, including traversal viability, hot/cold payload layout, insert/vacuum parity, and matched recall/latency/storage evidence against TurboQuant and PqFastScan.
64. `64-hnsw-quantized-codec-adapters.md` — companion to Task 63: extract a narrow HNSW-local codec adapter seam for TurboQuant/PqFastScan first so RaBitQ can plug into HNSW without a broad refactor or premature cross-AM codec trait.
65. `65-diskann-build-perf-vamana-core.md` — **complete** (2026-05-28, closeout `8e3109c25`): landed the six P0 single-process DiskANN/Vamana build fixes identified in `plan/design/diskann-build-performance.md` (O(N²) dedup removal, per-search bitset churn, linear-scan frontier, per-pivot pool bitsets, two-pass → growing-α single pass, rayon-parallel codec encode); final R32/L200 real10k build is 14.92s against the 16s gate, recall holds on real10k and synth10k, and parallel graph construction stays deferred to Task 65b.
66. `65b-diskann-build-parallel-graph-construction.md` — parallel Vamana graph construction follow-up after Task 65 lands the single-process algorithmic core; target real-10k build ≤3s on a 4-core host via shared neighbour cache + Postgres ParallelContext workers (matching HNSW's build_parallel.rs pattern) or a rayon stepping stone; load-bearing design call is determinism vs throughput on backlink races.
67. `66-rabitq-m5-neon-optimization.md` — close the NEON-side scoring gaps in `src/quant/rabitq.rs` for M5: arithmetic-dequant bits=8 NEON kernel covering the active `rabitq8`/`rabitq8ls`/`rabitq8c3`/`rabitq8c4` IVF variants, true batched bits=1/bits=8 scoring, software prefetch in the bandwidth-bound bits=1 path, and an M5-specific bf16 re-measurement; also lands the per-arch dispatch seam + differential-test scaffold that Task 67 (Intel) consumes.
68. `67-rabitq-intel-avx-optimization.md` — Intel AVX-512/AVX2 kernels for RaBitQ scoring (bits=1 VPOPCNTDQ + AVX2 popcount fallback, bits=4 nibble-unpack FMA, bits=8 arithmetic-dequant) plus batched paths; depends on Task 66's dispatch seam — this task must not edit the shared fan-out functions, only register new per-arch kernel slots; closes the silent x86 disadvantage in Task 60/63 RaBitQ benchmarks on Intel hosts.
69. `68-spire-build-perf-characterization.md` — **complete** (2026-05-30, closeout `reviews/task-68/008-closeout/`): characterized SPIRE build time at 10k/100k, landed the zero-replica leaf row fast path and top-graph distance cache, shelved deeper top-graph work below the continuation gate, and closed with reviewer-approved final build split, recall, determinism, and no-new-unsafe evidence.
70. `69-common-training-parallelism.md` — **complete** (2026-05-30, closeout `reviews/task-69/004-closeout/`): landed rayon-parallel k-means, grouped PQ4 training, and deterministic assignment fan-out in `src/am/common/training.rs`; reviewer-approved measurement shows 11.6x-13.7x release speedups at SPIRE/IVF-shaped training sizes, byte-equal outputs vs scalar references, and no `RAYON_NUM_THREADS=1` regression.
71. `70-diskann-scan-kernel-optimization.md` — **complete** (2026-05-31, closeout `reviews/task-70/012-final-measurement-docs/`): characterized the M5 `ec_diskann` scan latency split at L=64 / L=200, landed measured frontier/candidate-management wins, shelved negative P0 attempts with evidence, preserved recall floors, and updated the DiskANN M5 cross-engine row in `docs/benchmarks.md`.
72. `71-ivf-parallel-build.md` — set `amcanbuildparallel = true` on `ec_ivf`, mirror HNSW Task 26 / ADR-048 ConcurrentDsm pattern adapted to IVF's coarse-quantizer + posting-list structure; depends on closed Task 69 for the parallel training/assignment surface and coordinates with deferred Task 57 (IVF unsafe burndown) on `ec_ivf/build.rs`.
73. `72-spire-parallel-build.md` — post-Task-68 follow-on: set `amcanbuildparallel = true` on `ec_spire`, parallelize the remaining 38% heap-scan share at 100k, hold determinism via structural-hash equality across worker counts, and explicitly skip publish/object-store parallelism (crash-safety surface deferred); coordinates with active Task 30 phases on SPIRE recursion.
74. `73-spire-recall-characterization.md` — **complete** (2026-05-31, closeout `reviews/task-73/002-closeout/`, reviewer acceptance `reviews/task-73/003-completion-audit/feedback/2026-05-31-01-reviewer.md`, follow-up `reviews/task-73/004-reviewer-followup/`): local M5 and AWS Graviton suites reproduced the Task 68 default recall floor, showed SPIRE reaches 0.9975-1.0000 recall@10 at `tg128/b0`, and shelved default/routing changes because the quality point carries a large latency cost now tracked in `plan/design/spire-quality-defaults-followup.md`.
75. `74-spire-leaf-scan-overhead.md` — **complete** (2026-05-31, closeout `reviews/task-74/002-closeout/`, audit `reviews/task-74/003-completion-audit/`, AWS profiler attempt `reviews/task-74/004-aws-profiler-attempt/`, Intel profiler baseline and reviewer acceptance `reviews/task-74/005-intel-profiler-baseline/`): local M5 and AWS Graviton evidence confirm material SPIRE-vs-IVF overhead at matched recall; Intel-local perf/flamegraph evidence at nprobe 96 shows SPIRE p50 `137.9 ms` vs IVF p50 `37.8 ms`, with visible SPIRE self-time dominated by quantized scoring rather than routing/candidate orchestration. Reviewer accepted the Intel-local packet as satisfying the profiler gate and the no-Phase-2-slice closeout direction because identifiable SPIRE-specific orchestration is below the task's 10% stop-condition floor.
76. `75-spire-latency-routing-envelope.md` — **pending reviewer acceptance** (2026-05-31, diagnostic fix `reviews/task-75/004-diagnostic-fix-rerun/`, Phase 2 reissue `reviews/task-75/005-phase2-decision-after-diagnostic-fix/`, closeout reissue `reviews/task-75/006-closeout-after-diagnostic-fix/`): Intel-local routing-funnel evidence was rerun after fixing SQL diagnostics to use top-graph routing; high-recall SPIRE reaches recall@10 `0.9975` at tg96/tg128 and scans `15,506,227` candidates over 200 queries while only `5,000` survive to heap rerank, so Task 75 shelves routing slices and moves candidate materialization/scoring optimization to Task 77.
77. `76-spire-recall-default-pareto.md` — **complete** (2026-05-31, measurement `reviews/task-76/001-pareto-measurement/`, closeout `reviews/task-76/002-closeout/`, preset note `reviews/task-76/003-closeout-preset-note/`): Intel-local `ecaz bench suite` Pareto evidence covered 10k and 100k SPIRE/IVF/HNSW controls, found no 100k SPIRE default point that beats current defaults once high-recall latency and tails are considered, keeps SPIRE defaults unchanged because the canonical 1M TSV fixture was unavailable, and explicitly shelves the quality-preset reloption until broader corpus/AWS evidence or Task 77 candidate-cost improvements justify a durable API.
78. `77-spire-candidate-materialization-optimization.md` — proposed SPIRE optimization follow-up after Tasks 75/76: reduce high-recall candidate production/materialization cost without changing recursion semantics or defaults, starting with Intel-local candidate-cost attribution and only running AWS after a local slice preserves recall and clears the p50 win gate.

## Coordination rules

- Freeze binary datum layout before downstream work expands.
- Freeze `ProdQuantizer` scoring interfaces before SIMD work begins.
- Freeze page tuple and WAL helper APIs before build, vacuum, and insert proceed independently.
- Keep benchmark work off the critical path until correctness is stable.
- **Planner agent owns `am/cost.rs`, `am/explain.rs`, `am/stream.rs`.** Graph search agent owns `am/scan.rs`, `am/search.rs`. No overlapping file edits during D1.
- **D2 wiring touches `am/scan.rs`** — only start D2 after graph search agent completes A3/A4 and is no longer modifying scan.
- **Do not remove ADR-011** (`f64::MAX` cost gate) until A4 recall gate passes. This is the planner activation gate.
- Merge SIMD after A3 confirms scalar correctness.
