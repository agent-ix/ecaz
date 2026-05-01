# Task 29: DiskANN Initial Tuning Lane

Status: **landed on `main`** — Task 29/29a/29b/29c/29d are merged with local
PG18 release measurements recorded. Task 29e is a follow-up cleanup/evidence
packet, not a current landing blocker.
Owner: coder1 / runtime-index track

## Follow-up tasks

- **Task 29a — binary-sidecar prefilter swap** (landed on `main`).
  `plan/tasks/29a-diskann-binary-sidecar-prefilter.md`. Closed the
  scan-path recall ceiling. Recall@10 went from ~0.93 to 0.997 at
  default reloptions on real-10k. Latency cleanup followed in the
  same lane via heap-frontier and early-stop scan changes. Packets:
  `11096`, `11097`, `11098`.
- **Task 29b — cleanup and vacuum consistency** (landed on `main`).
  `plan/tasks/29b-diskann-cleanup-and-vacuum-consistency.md`.
  Wires the binary sidecar into vacuum-repair candidate scoring,
  finalizes the `ec_diskann.prefilter_kind` GUC as a production
  rollback knob, adds the missing pgrx test, verifies SIMD codegen
  on `hamming_xor_popcount`, and tightens code shape. Grouped-PQ
  stays — it is shared infrastructure with `ec_hnsw` / `ec_ivf` and
  remains the GUC-controlled rollback path. Packet: `11100`.
- **Task 29c — build performance** (landed on `main`; no Task 29d
  blocker opened). `plan/tasks/29c-diskann-build-perf.md`.
  Structured build timing showed the apparent ~492 s real-10k build
  was a debug/dev-installed extension artifact. The same head with a
  release-installed extension initially built the isolated real-10k
  DiskANN index in `79.238s`; the active-mask prune cleanup then
  improved that to `70.678s`. The remaining cost is Vamana graph
  construction, not tuple persistence or page writes. Reference
  `ec_hnsw` on the same table with `m=32`, `ef_construction=100`
  built in `5.23s`. Packets: `11101`, `11102`, `11104`.
- **Task 29d — pre-landing perf sweep** (landed on `main`).
  `plan/tasks/29d-diskann-pre-landing-perf-sweep.md`.
  Build heap-frontier release A/B stayed reverted (`11106`), L=64
  scan profiling found no safe default rerank-budget change (`11107`),
  and the build-distance SIMD change landed (`11108`). Final local PG18
  release readiness packet `11109` measured `ec_diskann` at 14.59 s
  build, 4,939,776 B index size, recall@10 `0.9965` to `0.9975`, and
  mean latency `7.80` to `9.34 ms` across L=64/128/200/400/800.

The landing-readiness packets are
`review/11099-task29-diskann-landing-readiness/`,
`review/11100-task29b-diskann-vacuum-prefilter-consistency/`,
`review/11104-task29c-prune-active-mask-profile/`,
`review/11105-task29-release-latency-refresh/`, and
`review/11109-task29d-final-readiness/`. Round-3 merge feedback in
`review/11105-.../feedback.md` has been addressed by Task 29d.

Task 29e is recorded in `review/11110-task29e-rerank-borrowed-simd/`.
The kept code cleanup is neutral for latency, and the rejected experiments are
not active follow-up work.

## Goal

Establish initial, reproducible build/recall/latency baselines for DiskANN on
the same corpus surfaces used by HNSW and IVF, then identify the first concrete
optimization and implementation slices for the DiskANN path.

DiskANN stayed separate from task 28. IVF landed first, and DiskANN landed as
its own first-class lane rather than being collapsed into IVF tuning.

## Hardware Baseline

Use the current local development environment for initial tuning and smoke
measurements. These numbers may guide implementation work, but they are not
publishable product claims.

Record enough metadata in every packet to make local measurements interpretable:

- machine / OS shape, CPU, memory, and storage location;
- compiler profile and extension SHA;
- PostgreSQL version and relevant settings;
- corpus manifest, row count, dimensionality, query count, and cache state.

Future product-claim benchmarks should move to a dedicated Graviton-class host.

## Phase 1: Reference Baselines

- Reuse the canonical DBPedia 990k/10k anchor and any smaller local smoke
  surfaces established by task 28.
- Measure HNSW current state only as a reference row.
- Add pgvectorscale DiskANN or another PostgreSQL DiskANN reference if
  installable without distorting the local benchmark environment.
- Capture build time, load time, index size, recall@10, p50/p95/p99 latency,
  cache state, memory high-water mark, and disk-read behavior where practical.

## Phase 2: DiskANN Candidate

- Stand up the ADR-034 `ec_diskann` candidate or a faithful reference harness
  before touching AM integration.
- Measure Vamana build cost, graph size, disk-read pattern, page-cache behavior,
  and compressed-code scoring throughput.
- Keep the first implementation batch-oriented and benchmarkable; live insert
  and vacuum semantics can follow after the build/search shape is justified.
- Gate on a repeatable result showing that disk-resident traversal has a
  credible path beyond the HNSW memory-resident ceiling.

## Phase 3: Initial Optimization

- Identify the strongest immediate DiskANN optimization slices based on measured
  recall/latency/build curves.
- Optimize the hot paths using the existing SIMD and quantizer surfaces.
- Add narrow review packets for each optimization claim with packet-local logs.
- Convert the measured findings into follow-up implementation tasks for DiskANN.

## Deferred Follow-Ups

- Larger structural low-L latency work that reduces heap visits or changes the
  exact-rerank storage path.
- Larger graph construction/layout changes if future hardware/product
  benchmarking requires build parity work.
- GPU-accelerated offline build/training from ADR-046.
- SPANN-style routing from ADR-035.

## Acceptance Criteria

- [x] Local initial-tuning review packets with complete reproducibility metadata.
- [x] A table comparing HNSW and DiskANN candidates on the same corpus and metric
  definitions.
- [x] A clear next-slice recommendation for DiskANN.
- [x] A note separating local tuning results from future Graviton-class product
  benchmark requirements.
