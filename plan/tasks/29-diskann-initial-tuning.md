# Task 29: DiskANN Initial Tuning Lane

Status: **pre-landing perf sweep in progress (29d)** — code merge-ready,
final perf items being run down on the same branch before merge
Owner: coder1 / runtime-index track

## Follow-up tasks

- **Task 29a — binary-sidecar prefilter swap** (LANDED on branch).
  `plan/tasks/29a-diskann-binary-sidecar-prefilter.md`. Closed the
  scan-path recall ceiling. Recall@10 went from ~0.93 to 0.997 at
  default reloptions on real-10k. Latency cleanup followed in the
  same lane via heap-frontier and early-stop scan changes. Packets:
  `11096`, `11097`, `11098`.
- **Task 29b — cleanup and vacuum consistency** (LANDED on branch).
  `plan/tasks/29b-diskann-cleanup-and-vacuum-consistency.md`.
  Wires the binary sidecar into vacuum-repair candidate scoring,
  finalizes the `ec_diskann.prefilter_kind` GUC as a production
  rollback knob, adds the missing pgrx test, verifies SIMD codegen
  on `hamming_xor_popcount`, and tightens code shape. Grouped-PQ
  stays — it is shared infrastructure with `ec_hnsw` / `ec_ivf` and
  remains the GUC-controlled rollback path. Packet: `11100`.
- **Task 29c — build performance** (LANDED on branch; no Task 29d
  blocker opened). `plan/tasks/29c-diskann-build-perf.md`.
  Structured build timing showed the apparent ~492 s real-10k build
  was a debug/dev-installed extension artifact. The same head with a
  release-installed extension initially built the isolated real-10k
  DiskANN index in `79.238s`; the active-mask prune cleanup then
  improved that to `70.678s`. The remaining cost is Vamana graph
  construction, not tuple persistence or page writes. Reference
  `ec_hnsw` on the same table with `m=32`, `ef_construction=100`
  built in `5.23s`. Packets: `11101`, `11102`, `11104`.
- **Task 29d — pre-landing perf sweep** (planned, blocks merge).
  `plan/tasks/29d-diskann-pre-landing-perf-sweep.md`. Three final
  perf items being run down on this branch before merge:
  (29d-1) build heap-frontier release-mode A/B to settle the
  round-2 deferred question — the same data-structure shape was a
  release-mode win on the scan side and a debug-mode regression on
  the build side, deserves a definitive release-mode answer;
  (29d-2) L=64 scan latency parity with pgvectorscale (currently
  9.19 ms vs 3.56 ms — the cleanest constant-factor signal in the
  comparison); (29d-3) DiskANN build performance attack against the
  pgvectorscale (5.82 s) and HNSW (5.23 s) references, stop
  condition at within 3× of the strongest reference. Each sub-task
  lands its own packet; final 29d readiness packet refreshes the
  full sweep before round-4 sign-off.

The current landing-readiness packets are
`review/11099-task29-diskann-landing-readiness/`,
`review/11100-task29b-diskann-vacuum-prefilter-consistency/`,
`review/11104-task29c-prune-active-mask-profile/`, and
`review/11105-task29-release-latency-refresh/`. Round-3 merge
feedback is in `review/11105-.../feedback.md` and tracks the
remaining pre-merge work as Task 29d.

## Goal

Establish initial, reproducible build/recall/latency baselines for DiskANN on
the same corpus surfaces used by HNSW and IVF, then identify the first concrete
optimization and implementation slices for the DiskANN path.

DiskANN is separate from task 28. IVF goes first; DiskANN remains a first-class
future work stream rather than being collapsed into the IVF tuning task.

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

- Production `ec_diskann` AM integration after the reference harness or early
  candidate justifies the build/search shape.
- Live insert and vacuum semantics for Vamana.
- GPU-accelerated offline build/training from ADR-046.
- SPANN-style routing from ADR-035.

## Acceptance Criteria

- Local initial-tuning review packets with complete reproducibility metadata.
- A table comparing HNSW and DiskANN candidates on the same corpus and metric
  definitions.
- A clear next-slice recommendation for DiskANN.
- A note separating local tuning results from future Graviton-class product
  benchmark requirements.
