# Task 94: Grouped-PQ / PqFastScan Block Kernel Family (All AMs × All ISAs)

Status: in review — G4 pass only (2026-06-10, packet 028 AC5 closeout
approved per `reviews/task-94/028-release-ac5-rerun/feedback/2026-06-10-01-reviewer.md`:
"the Task 94 F8 slice has complete local evidence; the only remaining
Task 94 item is the deferred Graviton 4 pass", sequenced after Task 102.
All implementation and local evidence merged to main via PRs #19/#22–#24.
Status line refreshed 2026-06-11; previously stale at the 2026-06-09
wording.)
Owner: coder (to be assigned). Phase III parallel — multiple coders OK across Tasks 93–98.
Priority: 2 (highest documented kernel-win ROI in Phase III)

## Reopened scope (2026-06-10): F8 shuffle-repack kernel slice

Per operator decision, one more in-task implementation slice is
required before the Graviton 4 / closeout pass. Measured per-candidate
rates from this task's own packet 025 counters (AVX2 kernel ~98
ns/cand vs scalar ~110 ns/cand on DiskANN cells) show the f32-gather
first-landing kernel is gather-bound — the Phase 1 design deferred
byte/shuffle LUT repacking "unless measurements require it", and they
now do. The slice:

- AVX2: FAISS PQ4-FastScan-style `_mm256_shuffle_epi8` with the
  16-entry LUT held in registers (the original task-file design).
- NEON: `vqtbl1q_u8` sibling.
- SVE2: `tbl`-based sibling — the current SVE kernel has the same
  scalar-gather shape as the gather-bound AVX2 one. Since the
  Graviton 4 pass deliberately runs after this slice so ARM evidence
  reflects final kernel shape, the SVE2 repack must land in this
  slice too, or be explicitly deferred in the slice's packet with the
  G4 evidence annotated as measuring the gather-shape SVE2 kernel.
- Parity: the scalar f32-LUT reference remains the strict bit-exact
  anchor (designed to survive this repack); the repacked kernels grade
  under the ADR-076 tolerance pair, or bit-exact if per-candidate
  accumulation order is preserved.
- Evidence: re-run the packet 025 local matrix; IVF is the primary
  beneficiary (full block coverage already); DiskANN gains require
  Task 101's width-cascade dispatch in combination.
- Source of record:
  `reviews/task-99/000-pre-closeout-architecture-review/feedback/`
  (finding F8).

The Graviton 4 pass executes after this slice and Task 101, so ARM
evidence is collected once against the final kernel shape. (Stale
pointer fixed 2026-06-11: this previously cited a "runbook (packet
027)", but ordinal 027 was consumed by the local latency-width rerun
and no Task 94 runbook packet was ever written. The G4 pass now rides
the Task 99 profile trip — see
`reviews/task-99/002-profile-suiteconfig/artifacts/t99-profile-design.md`
— whose grouped-PQ cells (IVF pq_fastscan batch-on/off,
DiskANN prefilter_kind=grouped_pq batch-on/off, with counters) are this
task's closing G4 evidence, annotated as measuring the gather-shape
SVE2 kernel if the SVE repack remains deferred.)

## Why

Grouped-PQ (a.k.a. PqFastScan) is the **canonical block-kernel
quant** in the literature. FAISS PQ4-FastScan papers report 3–8×
scoring-share wins over scalar PQ across IVF and DiskANN-class
indexes. The kernel structure — per-group LUT lookup + accumulate
across groups, with the LUT held in SIMD registers and 32
candidates scored per inner-loop iteration — is the prototype
for every other LUT-based block kernel in this matrix.

Current scoring paths in tree:

- IVF: `IvfQuantizer::score_ip_from_parts` on `IvfPreparedQuery::
  PqFastScan` branch — per-candidate `score_ip_from_parts` over
  grouped codes.
- DiskANN: `DiskannPreparedPrefilter::GroupedPq::score` per node
  during Vamana scan.

IVF and DiskANN posting/page batches are typically 64–256
candidates — well above the 32-block threshold. This is the
highest-ROI cell in the Phase III matrix.

## Scope

### In scope

Implementation note: Phase 1 reviewer feedback approved
`src/quant/grouped_pq_block/` as the module path, replacing the original
`pq_fastscan32/` wording while retaining PqFastScan terminology for the
storage format.

1. **Scalar block kernel** at `src/quant/grouped_pq_block/scalar.rs`
   following the FAISS PQ4-FastScan algorithm (per-group SIMD
   LUT lookup pattern, in scalar form as the bit-equal reference).
2. **NEON variant** under `src/quant/grouped_pq_block/neon.rs`.
   The first landing uses the approved f32 LUT gather / vector accumulate
   shape; packed-table repacks remain follow-up-only if measurements show
   gather stalls dominate. Validate on Graviton 4 with the NEON path forced,
   plus any cheaper ARM sanity host if available.
3. **SVE/SVE2 variant** under `src/quant/grouped_pq_block/sve.rs`.
   The first landing uses a vector-length-agnostic SVE accumulation helper
   over f32 LUT gathers. Report a width-specific label such as `sve2-128`
   only when measured at runtime.
4. **AVX2 variant** under `src/quant/grouped_pq_block/avx2.rs`.
   The first landing uses `_mm256_i32gather_ps` against the canonical f32 LUT
   rows; byte/shuffle repacks are follow-up-only if justified by measurement.
5. **`QuantCodec` registration** in IVF + DiskANN grouped-PQ
   impls. Width-based gating per ADR-076.
6. **Per-(AM × quant × ISA) measurement** on real10k / 50k /
   100k corpora for IVF + DiskANN grouped-PQ surfaces.
7. **Recall byte-equal at bench level** at every cell. Scalar
   block kernel asserts bit-equal vs the per-candidate
   reference; SIMD variants ULP-tolerant per ADR-076.
8. **Per-(AM × ISA) closeout matrix.**

### Out of scope

- New grouped-PQ AM coverage (e.g., grouped-PQ on HNSW or SPIRE
  if not present). Cover only IVF + DiskANN where grouped-PQ
  already exists.
- HNSW traversal-level grouped-PQ batching. HNSW has a grouped-PQ
  `QuantCodec` batch override for codec-surface parity tests, but the
  production greedy-search path scores one search code at a time through
  `score_grouped_search_code_result`, so real scans are expected to remain
  per-candidate scalar and not emit `surface=hnsw, quant=grouped_pq`
  kernel rows until a follow-up adds a natural traversal batch boundary.
- Codebook training optimization (Task 23 territory).
- AVX-512 variant.

## Acceptance criteria

1. `src/quant/grouped_pq_block/` module live with scalar + NEON +
   SVE + AVX2 per Task 92 convention.
2. IVF + DiskANN grouped-PQ scoring routes through `QuantCodec::
   <batch method selected by Task 91>` and dispatches to the kernel
   for batches ≥ 32.
3. Recall byte-equal at every cell.
4. ≥ 2× scoring-share latency on each ISA per AM. Per-ISA stop
   condition < 1.5× → document and continue.
5. End-to-end p50/p95/p99 measured; no regression beyond ULP noise.
6. Existing `pg_test` surfaces for grouped-PQ on IVF + DiskANN
   pass.
7. No new `unsafe` outside ISA module boundary; full safety doc
   on intrinsic-using modules.
8. Per-AM closeout matrix in `reviews/task-94/.../artifacts/`.

## Phases

### Phase A — Scalar block kernel + layout audit + scalar-baseline measurement

- Land `grouped_pq_block/scalar.rs` following the FAISS PQ4-FastScan
  algorithm shape. Per-group LUT lookup + group-wise accumulation
  across all 32 lanes.
- Audit IVF and DiskANN grouped-PQ code packing, group ordering,
  LUT signedness, and accumulation range before implementing SIMD.
  SIMD kernels must match the audited layout rather than assuming
  FAISS-compatible nibble packing.
- Route IVF + DiskANN grouped-PQ scoring through `QuantCodec`.
- Real10k baseline + kernel-on/off counters on both AMs.

### Phase B — NEON variant + Graviton ARM measurement

- Land `grouped_pq_block/neon.rs` using the approved first-pass f32 LUT
  gather / vector accumulate shape.
- Graviton 4 measurement with SVE disabled or the NEON dispatch path
  forced. A cheaper ARM host may be used only as supplemental sanity
  evidence.

### Phase C — SVE variant + Graviton 4 measurement

- Land `grouped_pq_block/sve.rs` using vector-length-agnostic SVE/SVE2
  accumulation. This is where Graviton 4's vector width may pay off most;
  record the measured vector length in artifacts.
- AWS Graviton 4 measurement; snapshot + destroy per memory.

### Phase D — AVX2 variant + Intel desktop measurement

- Land `grouped_pq_block/avx2.rs` using the approved first-pass f32 LUT
  gather path.
- Intel desktop measurement.

### Phase E — Per-(AM × ISA) closeout matrix + status flip

- Aggregate matrix.
- Scoring-share per ISA per AM with ≥2× gate call.
- End-to-end deltas with attribution.
- Status flip to `complete`.

## Per-AM validation gate

Same as Task 93. For each (AM × corpus) cell, kernel-on vs
kernel-off:

1. Recall byte-equal at bench level.
2. Scoring-share latency improves ≥ 2× per ISA.
3. End-to-end p50/p95/p99 improves or stays within ULP noise.
4. Storage unchanged.
5. `pg_test` surfaces pass.

## Stop conditions

- If per-ISA scoring-share < 1.5× on a touched AM: document, do
  not back out.
- If grouped-PQ algebra requires multiple group counts (current
  source has `group_count` parameter), parameterize the kernel
  for the most common group counts (typically 8, 16, 32). If a
  particular `group_count` configuration doesn't fit the kernel
  cleanly, fall back to scalar for that configuration and
  document.
- Recall byte-equality failure → BLOCK and triage.

## Coordination

- **Depends on Task 91 Phase 5** (DiskANN migration onto
  `QuantCodec`). IVF grouped-PQ trait registration may need Task
  91 Phase 2 IVF retouch first, depending on packet 008's
  grouped-PQ limit.
- **Depends on Task 92** infrastructure.
- **Parallel with Tasks 93, 95–98.** Highest-ROI Phase III task —
  prioritize coder assignment.
- **Consumed by Task 99.**

## References

- Task 15 (PqFastScan first-class)
- Task 23 (LSQ codebook refinement — grouped codebook prior art)
- Task 60 (DiskANN RaBitQ — grouped-PQ predecessor on DiskANN)
- ADR-030 (FastScan grouped subvector scoring)
- ADR-076 (universal block kernel pattern — Task 92)
- FAISS PQ4-FastScan paper (André et al., "Accelerated nearest
  neighbor search with quick ADC") — primary literature reference

## Estimated size

Medium-large. 5–8 weeks for one coder. The algorithm is the most
heavily documented kernel in the literature, but the AM-side
integration (grouped codebook model state ownership — Task 87
packet 008's known limit) needs to be cleanly handled at the
`QuantCodec` trait boundary first.
