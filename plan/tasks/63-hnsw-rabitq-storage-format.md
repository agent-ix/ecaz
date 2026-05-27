# Task 63: HNSW RaBitQ Storage Format

Status: **implementation landed; publishable benchmark decision pending**

Add a first-class RaBitQ storage format to `ec_hnsw`, or produce an
evidence-backed design decision explaining why HNSW should stay limited to
TurboQuant and PqFastScan. This is separate from Task 62: Task 62 optimizes the
existing HNSW formats, while this task decides whether RaBitQ belongs on HNSW
at all and implements it if the gate passes.

## Goal

Give HNSW a low-storage RaBitQ option comparable to the existing IVF/SPIRE
RaBitQ paths, while preserving HNSW's graph traversal semantics and recall
expectations. The task must answer:

- whether RaBitQ codes are accurate enough for HNSW greedy traversal at useful
  `ef_search` values;
- whether the storage reduction is large enough to justify another HNSW
  on-disk format;
- whether RaBitQ-on-HNSW needs a rerank/sidecar layout different from IVF and
  SPIRE; and
- whether this work should land before or after the Task 62 PqFastScan HNSW
  optimization pass.

## Original State

- `ec_hnsw` supports `storage_format = 'turboquant' | 'pq_fastscan'`.
- `ec_ivf` and `ec_spire` support `storage_format = 'rabitq'`.
- Task 60 tracks RaBitQ for `ec_diskann`; it is not an HNSW task.
- Task 62 tracks full HNSW Graviton optimization for existing formats and must
  not silently absorb this storage-format expansion.
- Task 64 is the companion HNSW codec-adapter extraction task. Use it to
  disentangle TurboQuant/PqFastScan payload, metadata, and scorer seams before
  or alongside this task; Task 63 remains responsible for the actual RaBitQ
  format and benchmark gate.

## Current State

The HNSW RaBitQ implementation has landed on branch
`task/60-diskann-rabitq`:

- `ec_hnsw` accepts `storage_format = 'rabitq'`.
- HNSW RaBitQ uses the shared `src/quant/rabitq.rs` quantizer/scorer.
- HNSW stores 1-bit RaBitQ search codes plus scalar rerank payloads rather than
  forking a separate HNSW-only RaBitQ kernel.
- Build, scan, live insert, delete, vacuum, and post-vacuum scans have PG18
  lifecycle smoke evidence in `reviews/task-63/007-*` through `010-*`.
- Task 64's HNSW-local codec adapter is the integration seam; no cross-AM codec
  trait was introduced.
- Follow-up packets `reviews/task-63/011-*` through `022-*` keep the benchmark
  handoff, user-facing docs, HNSW V4 RaBitQ fixture/upgrade matrix, reloption
  help/spec text, and common RaBitQ byte-LUT allocation audit aligned with the
  landed implementation.

The remaining completion gate is benchmark evidence and decision recording:

- run the checked-in HNSW-only `ecaz bench suite` configs at
  `benchmarks/task63-hnsw-rabitq-format/suite.json` on the newer Intel host
  and `benchmarks/task63-hnsw-rabitq-format/suite-m5.json` on the m5 laptop;
- report 50k and 100k recall@10, p50/p95/p99 latency, build time, and storage
  against TurboQuant and PqFastScan at matched `ef_search`;
- record the recommended RaBitQ HNSW operating point, or mark the format
  experimental/shelved if the measured recall/storage/latency tradeoff is not
  useful.

## Design Gate

Before implementing a new HNSW on-disk format, create a design/measurement
packet under `reviews/task-63/` or `benchmarks/task63-hnsw-rabitq-design/`
that answers:

1. **Traversal viability.** Can RaBitQ approximate scores drive HNSW layer
   descent and layer-0 expansion without unacceptable recall loss?
2. **Rerank strategy.** Is heap-f32 rerank required for final top-k, and at
   what candidate width?
3. **Payload layout.** Can HNSW reuse the existing RaBitQ payload encoder and
   query scorer, or does it need a HNSW-specific hot/cold tuple layout?
4. **Storage win.** What index-size reduction is expected versus
   `pq_fastscan` and TurboQuant at 100k and 1M?
5. **Implementation risk.** Which insert, vacuum, and page-format invariants
   need new coverage?

Do not implement the storage format until the packet recommends proceeding.

## Scope If Gate Passes

1. **Reloption.** Extend `ec_hnsw` storage-format parsing to accept
   `storage_format = 'rabitq'`. Keep the default unchanged.
2. **Metadata and page format.** Add an explicit HNSW metadata discriminator
   for RaBitQ. Existing TurboQuant and PqFastScan indexes must continue to load
   unchanged.
3. **Build path.** Encode HNSW graph payloads with the shared RaBitQ encoder.
   Reuse the existing quantizer implementation; do not fork a separate RaBitQ
   kernel.
4. **Scan path.** Add RaBitQ query preparation and scoring to HNSW traversal.
   Preserve existing HNSW scan behavior for TurboQuant and PqFastScan.
5. **Insert path.** Support post-build insert into RaBitQ HNSW indexes, or
   explicitly reject live insert with a documented reason and separate follow-up
   task. Silent partial support is not acceptable.
6. **Vacuum path.** Support dead tuple cleanup and page compaction for RaBitQ
   HNSW indexes, or explicitly gate the feature behind a no-vacuum limitation
   until parity lands.
7. **Tests.** Add format-specific build, scan, insert, and vacuum coverage.
8. **Bench packet.** Measure recall, latency, build time, and storage against
   TurboQuant and PqFastScan on at least 50k and 100k; include 1M if the host
   profile is already available.

## Non-Goals

- Do not change the default HNSW storage format.
- Do not optimize HNSW generally; Task 62 owns that lane.
- Do not optimize IVF/SPIRE RaBitQ in this task.
- Do not create mixed-format HNSW indexes. One index uses one storage format.
- Do not introduce a new RaBitQ algorithm variant unless the design packet
  proves the existing shared implementation is insufficient.

## Acceptance Criteria

Minimum to land implementation after the design gate:

- `CREATE INDEX ... USING ec_hnsw WITH (storage_format = 'rabitq')` builds and
  scans on real-corpus fixtures.
- Existing TurboQuant and PqFastScan HNSW indexes remain backward-compatible.
- Recall@10, p50/p95/p99 latency, build time, and storage are reported against
  TurboQuant and PqFastScan at matched `ef_search` values.
- The packet states the recommended operating point for RaBitQ HNSW, or states
  that the format should remain experimental/shelved.
- Insert and vacuum behavior is either fully supported or explicitly rejected
  with tests proving the rejection is clear and safe.

## Stop Conditions

- Stop after the design packet if RaBitQ traversal recall is not competitive at
  any useful storage/latency point.
- Stop if the required page-format change conflicts with active Task 42
  on-disk format invariants; coordinate there before proceeding.
- Stop before implementation if Task 62 shows PqFastScan HNSW already covers
  the target storage/latency envelope well enough that another HNSW format is
  not justified.
