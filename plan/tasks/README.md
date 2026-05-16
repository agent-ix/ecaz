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
29. `29-diskann-initial-tuning.md` — DiskANN Task 29/29a/29b/29c/29d landed on `main`; 29e is recorded as follow-up cleanup/evidence, not a current blocker.
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
31. `31-ivf-m5-optimization.md` — first-priority M5 optimization lane for landed IVF: refresh local baselines, classify the dominant bottleneck, and land one measured scan/scoring/churn optimization at a time.
32. `32-diskann-m5-optimization.md` — second-priority M5 optimization lane for landed DiskANN: refresh Task 29d baselines, profile low-L scan latency, and pursue targeted constant-factor wins without lowering recall floors.
33. `33-hnsw-m5-optimization.md` — third-priority M5 optimization lane for HNSW: refresh reference worker curves, then decide between direct DSM ingestion, offline/staged build, or narrow scan/build hot-path work.
34. `34-comprehensive-hardening.md` — local-first expansion of the ECAZ/SPIRE hardening stack: cargo-audit/deny/vet, Miri, cargo-careful, sanitizers, fuzzing, SQLsmith, Loom/Shuttle, Kani/Flux/MIRAI, Rudra, and unsafe-surface reporting.

## Coordination rules

- Freeze binary datum layout before downstream work expands.
- Freeze `ProdQuantizer` scoring interfaces before SIMD work begins.
- Freeze page tuple and WAL helper APIs before build, vacuum, and insert proceed independently.
- Keep benchmark work off the critical path until correctness is stable.
- **Planner agent owns `am/cost.rs`, `am/explain.rs`, `am/stream.rs`.** Graph search agent owns `am/scan.rs`, `am/search.rs`. No overlapping file edits during D1.
- **D2 wiring touches `am/scan.rs`** — only start D2 after graph search agent completes A3/A4 and is no longer modifying scan.
- **Do not remove ADR-011** (`f64::MAX` cost gate) until A4 recall gate passes. This is the planner activation gate.
- Merge SIMD after A3 confirms scalar correctness.
