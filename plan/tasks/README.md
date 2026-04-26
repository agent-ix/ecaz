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

11. `11-planner.md` — D1/D2 substantially complete on `main`; remaining follow-ons are measurement and optional parallel-scan callbacks

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
18. `18-parallel-index-scan.md` — executes ADR-040: `amcanparallel=true`, shared top-K coordinator, per-worker beams, ef_search split with overlap
19. `19-pg18-completion.md` — executes ADR-016/017: flip PG18 primary-target, activate amgettreeheight / EXPLAIN hook / pgstat-kind / ReadStream, drop PG14–16
20. `20-opq-rotation.md` — executes ADR-036: OPQ as alternative transform front-end for PqFastScan, +10–20% recall per byte, zero scan-kernel change
21. `21-simd-modernization.md` — executes ADR-039 + task-08 hot-path follow-up: AVX-512 specializations and ARM SVE/SVE2 backend under existing runtime dispatch
22. `22-additive-residual-quantization.md` — executes ADR-037: **evaluate-gated** feasibility study of AQ / RVQ as PqFastScan successor; three decision gates, shelf-on-fail OK
23. `23-lsq-codebook-refinement.md` — executes ADR-038: drop-in k-means replacement, +2–5% recall, no wire format change, low priority fill-in
24. `24-post-native-build-storage-and-lever4-followons.md` — post-ADR-042 follow-up: reopen ADR-044 on a stable native builder, carry forward the `EXTENDED` / `MAIN` build-collapse bug, and close the lever-4 `ef_search` matrix before any persisted-default decision
25. `25-rabitq-quantizer.md` — executes ADR-045 Stage 1: standalone RaBitQ quantizer + offline recall study; **research gate** for SymphonyQG Stages 2–3; shelvable as a clean null result if recall fails
26. `26-parallel-index-build.md` — parallel `CREATE INDEX` / `REINDEX` for `ec_hnsw` (propagating to `ec_diskann`); shares DSM/slot primitives with task 18, so Phase 2+ is gated on task 18 merge; ADR TBD
27. `27-symphony-access-method.md` — executes ADR-045 Stages 2–3: `symphony` AM with quantization-aware pruning + out-degree padding (Stage 2), then no-rerank query path (Stage 3); **gated on task 25's RaBitQ recall study passing**
28. `28-ivf-diskann-benchmark-optimization.md` — pivots the next scale research lane from more HNSW parallel-build tuning to real Graviton-class IVF/DiskANN build, recall, and latency baselines, then optimizes the stronger candidate before reopening frontier research on top.

## Coordination rules

- Freeze binary datum layout before downstream work expands.
- Freeze `ProdQuantizer` scoring interfaces before SIMD work begins.
- Freeze page tuple and WAL helper APIs before build, vacuum, and insert proceed independently.
- Keep benchmark work off the critical path until correctness is stable.
- **Planner agent owns `am/cost.rs`, `am/explain.rs`, `am/stream.rs`.** Graph search agent owns `am/scan.rs`, `am/search.rs`. No overlapping file edits during D1.
- **D2 wiring touches `am/scan.rs`** — only start D2 after graph search agent completes A3/A4 and is no longer modifying scan.
- **Do not remove ADR-011** (`f64::MAX` cost gate) until A4 recall gate passes. This is the planner activation gate.
- Merge SIMD after A3 confirms scalar correctness.
