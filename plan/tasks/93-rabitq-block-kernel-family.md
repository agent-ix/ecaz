# Task 93: RaBitQ Block Kernel Family (All AMs × All ISAs)

Status: complete (2026-06-10, closeout `reviews/task-93/007-closeout-status/`,
reviewer-approved with owner lane decisions recorded 2026-06-10: the
Graviton SVE lane and the Intel AVX2 measurement are both explicitly
deferred into Task 99's cross-ISA profile. Scalar + NEON measured on all
three AMs x real10k/50k/100k with recall byte-equal and every gate passed;
AVX2 code landed bit-equal-by-construction with production primitives; SVE
hosts route through NEON with truthful isa attribution. Task 99 carries
the IVF ~99% 32-block-coverage datum for the SVE decision)
Owner: coder (to be assigned). Phase III parallel — multiple coders OK across Tasks 93–98.
Priority: 2 (Phase III block kernel family — RaBitQ)

## Why

RaBitQ is shipping production scoring on IVF (Task 51), DiskANN
(Task 60), and HNSW (Task 63). The current scoring path is
per-candidate scalar:

- IVF: `PreparedEstimator::estimate_ip_scalar_only(payload)`
  per posting-list row.
- DiskANN: `DiskannPreparedPrefilter::RaBitQ::score` per node.
- HNSW: RaBitQ prepared scorer per neighbor.

IVF already has a `score_ip_bits1_batch_from_payloads` helper
proving the batch shape works on RaBitQ — that's the scalar
reference for the block kernel.

RaBitQ scoring is `popcount(query_words XOR code_words) +
scalar_correction(per-candidate metadata)`. Block-kernel
benefit is direct: amortize query_words load across 32 candidates,
batch-popcount, batch-scalar-correct. Literature wins for SIMD
popcount kernels are 4–8× over scalar.

## Scope

### In scope

1. **Scalar block kernel** at `src/quant/rabitq32/scalar.rs`,
   covering RaBitQ 1-bit (the primary deployed bit-width).
   Multi-bit RaBitQ variants (Task 60/66/67 paths) covered via
   a parameterized kernel if the algebra is straightforward;
   otherwise scoped as follow-up.
2. **NEON variant** at `src/quant/rabitq32/neon.rs` using NEON
   `cnt` (vector popcount) + `veor` (xor). Validate on Graviton 4
   with the NEON path forced, plus any cheaper ARM sanity host if
   available.
3. **SVE variant** at `src/quant/rabitq32/sve.rs` using SVE
   `cnt` + predicate masks for tail handling. Report as SVE-256
   only when the measured runtime vector length is 256 bits.
4. **AVX2 variant** at `src/quant/rabitq32/avx2.rs` using
   AVX2 `_mm256_xor_si256` plus an AVX2-compatible popcount
   strategy such as nibble-LUT/`pshufb` + `_mm256_sad_epu8`.
   VPOPCNTDQ is AVX-512-family only and belongs in a future
   AVX-512 variant, not the AVX2 gate.
5. **`QuantCodec` registration** of the kernel in each AM's RaBitQ
   impl. Width-based gating: `batch.len() >= 32` routes to the
   kernel; smaller batches use scalar fallback.
6. **Per-(AM × quant × ISA) measurement** across real10k / 50k /
   100k corpora on every AM that exposes RaBitQ:
   - IVF (Task 51 surfaces);
   - DiskANN (Task 60 surfaces);
   - HNSW (Task 63 surfaces).
7. **Recall byte-equal at bench level** at every cell vs the
   pre-kernel baseline. Scalar kernel asserts bit-equal vs
   `score_ip_bits1_batch_from_payloads` reference; SIMD
   variants ULP-tolerant per ADR-076.
8. **Per-AM closeout** with the per-ISA scoring-share table.

### Out of scope

- Storage format changes. RaBitQ on-disk layout stays as-is.
- New RaBitQ AM coverage (e.g., SPIRE RaBitQ if not present).
  Cover only the AMs that already expose RaBitQ.
- AVX-512 variant. Follow-up after Task 99 if measurement
  justifies.
- TQ-no-QJL-4-bit kernel (Task 87) — already shipped.

## Acceptance criteria

1. `src/quant/rabitq32/` module live with scalar + NEON + SVE
   + AVX2 variants per the Task 92 convention.
2. Each AM's RaBitQ scoring path routes through `QuantCodec::
   <batch method selected by Task 91>` and dispatches to the
   kernel for batches ≥ 32.
3. Recall byte-equal at every (AM × corpus) cell vs pre-kernel
   baseline.
4. ≥ 2× scoring-share latency on the kernel path vs the scalar
   reference (off-path counter from Task 92), per ISA per AM
   where measured. Per-ISA stop condition < 1.5× → document and
   continue, do not back out.
5. End-to-end p50/p95/p99 measured at every cell; no regression
   beyond ULP-attributable noise.
6. Existing `pg_test` surfaces for RaBitQ on IVF, DiskANN, HNSW
   pass.
7. No new `unsafe` outside the ISA module boundary. SIMD
   intrinsics land with `# Safety` doc + scalar differential
   coverage per `feedback_dont_defer_safety_fixes`.
8. Per-AM closeout matrix in `reviews/task-93/.../artifacts/`.

## Phases

### Phase A — Scalar block kernel + scalar-baseline measurement

- Land `rabitq32/scalar.rs`. Bit-equal vs
  `score_ip_bits1_batch_from_payloads`.
- Route IVF, DiskANN, HNSW RaBitQ scoring through
  Task 91's selected `QuantCodec` batch method + scalar kernel
  dispatch.
- Real10k baseline measurement on each AM. Kernel-on vs
  kernel-off cells via Task 92 counters.

### Phase B — NEON variant + Graviton 4 forced-NEON measurement

- Land `rabitq32/neon.rs` using NEON popcount.
- Same matrix on Graviton 4 with SVE disabled or the NEON dispatch
  path forced. A cheaper ARM host may be used only as supplemental
  sanity evidence.
- ULP tolerance ≤ 4 ULP vs scalar reference. Recall byte-equal at
  bench level.

### Phase C — SVE variant + Graviton 4 measurement

- Land `rabitq32/sve.rs` using vector-length-agnostic SVE `cnt`
  + predication.
- AWS Graviton 4 measurement run. Per memory rules, snapshot +
  destroy the bench host immediately after measurement.

### Phase D — AVX2 variant + Intel desktop measurement

- Land `rabitq32/avx2.rs`.
- Intel desktop measurement (local bench host).

### Phase E — Per-(AM × ISA) closeout matrix + status flip

- Aggregate matrix across all AMs × all ISAs × all corpora.
- Scoring-share per ISA per AM with the ≥2× gate call.
- End-to-end deltas with attribution.
- Status flip to `complete` referencing the closeout packet.

## Per-AM validation gate

For each (AM × corpus) cell, kernel-on vs kernel-off:

1. Recall byte-equal at bench level.
2. Scoring-share latency improves ≥ 2× (per ISA where available).
3. End-to-end p50/p95/p99 improves or stays within ULP noise.
4. Storage unchanged (no on-disk format change in scope).
5. `pg_test` surfaces for the AM pass.

## Stop conditions

- If per-ISA scoring-share win is < 1.5× on a touched AM:
  document the measured factor + scoring-share total share of
  query time. Don't back out the kernel (kernel still preserves
  recall and is on a path consumed by Task 99 closeout).
- If recall byte-equality fails at any cell, BLOCK the slice and
  triage. Likely points: ULP tolerance exceeded for that quant's
  algebra, or scalar reference miscomputes a tail case.
- If multi-bit RaBitQ algebra is not parameterizable cleanly,
  scope down Task 93 to 1-bit and open a follow-up for multi-bit.

## Coordination

- **Depends on Task 91 Phase 4 (HNSW migration)** and **Phase 5
  (DiskANN migration)** for the trait surface those AMs need to
  register against. IVF is on the trait via Task 87 packet 008 +
  Task 91 Phase 2 retouch.
- **Depends on Task 92** for ADR-076, counter surface, ISA
  detection helper, module convention, bench suite cross-quant
  axis.
- **Parallel with Tasks 94, 95, 96, 97, 98.** RaBitQ kernel
  shares popcount structure with Hamming (Task 95). Worth
  coordinating one-shared-popcount-helper if Task 95 lands
  first; otherwise factor common code out at Task 99.
- **Consumed by Task 99** in the cross-(AM × quant × ISA) matrix.

## References

- Task 51 (IVF RaBitQ)
- Task 60 (DiskANN RaBitQ)
- Task 63 (HNSW RaBitQ)
- Task 66 (RaBitQ M5 NEON optimization — predecessor NEON work
  to reference)
- Task 67 (RaBitQ Intel AVX optimization — predecessor AVX work)
- Task 86 packet 002 (block-kernel transferability matrix)
- ADR-031 (RaBitQ binary prefilter)
- ADR-076 (universal block kernel pattern — Task 92)

## Estimated size

Medium. 4–6 weeks for one coder. Phases A–B can land within the
first two weeks; SVE (Phase C) is the slowest because cloud
bench setup overhead applies; AVX2 + closeout in the final 1–2.
Multi-bit RaBitQ may extend size if in scope.
