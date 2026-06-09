# Task 97: TurboQuant QJL Block Kernel Family (Gamma + Residual Signs)

Status: proposed (2026-06-08)
Owner: coder (to be assigned). Phase III parallel.
Priority: 2 (QJL variant; per-candidate metadata complicates kernel)

## Why

TurboQuant QJL is the gamma-aware path: per-candidate scoring
adds a `gamma` scalar correction plus residual sign bits
applied to the LUT result. Current scoring goes through
`score_ip_from_parts(prepared, gamma, payload)` per-candidate.
No batched kernel.

The block kernel pattern still applies — LUT lookup amortizes
across 32 candidates — but per-candidate gamma and residual
signs must be threaded through the inner loop as side data.
Literature suggests 2–4× SIMD wins; less than no-QJL because
of the per-candidate correction overhead.

QJL is used where the recall vs storage trade-off favors
carrying gamma (recall-critical surfaces). Coverage in tree:
SPIRE TurboQuant gamma-aware path (no-QJL 4-bit lane is the
LUT path; QJL is the gamma-aware fallback). HNSW gamma-aware
fallback. IVF generic TurboQuant branch.

## Scope

### In scope

1. **Scalar block kernel** at `src/quant/qjl32/scalar.rs` with
   per-candidate gamma + residual sign side data.
2. **NEON variant** using NEON `tbl` for LUT + scalar
   multiply-add for gamma correction.
3. **SVE variant** using vector-length-agnostic SVE `tbl` +
   predicated FMA for gamma + sign mask. Report as SVE-256 only
   when the measured runtime vector length is 256 bits.
4. **AVX2 variant** using `_mm256_shuffle_epi8` + AVX2 FMA.
5. **`QuantCodec` registration** on each AM's QJL/gamma-aware
   scoring path. Side data carried through `CandidateMeta::
   Gamma(f32)` or `CandidateMeta::GammaAndResidualSigns {...}`
   as Task 91 trait defines.
6. **Per-(AM × ISA) measurement** on surfaces with QJL
   storage / gamma-aware scoring.
7. **Recall byte-equal** per ADR-076. QJL kernel ULP tolerance
   needs careful audit because per-candidate FMA reordering
   can compound across lanes.
8. **Per-AM closeout matrix.**

### Out of scope

- Pure no-QJL TurboQuant paths — those are Task 87 (4-bit) or
  Task 96 (2-bit).
- Storage format work for QJL adoption.

## Acceptance criteria

1. `src/quant/qjl32/` module live with scalar + NEON + SVE
   + AVX2.
2. Each AM with QJL/gamma-aware scoring routes through
   Task 91's selected `QuantCodec` batch method with
   `CandidateMeta::Gamma*` side data.
3. Recall byte-equal at every cell.
4. ≥ 2× scoring-share per ISA per AM is the target. Because QJL's
   per-candidate correction reduces the ceiling vs no-QJL, ≥1.8×
   is acceptable without a Stop Condition; 1.5×–1.8× is documented
   as partial; <1.5× triggers the per-ISA Stop Condition process.
5. End-to-end no regression beyond noise.
6. `pg_test` surfaces for QJL-using AMs pass.
7. Safety docs on FMA-using paths.
8. Per-AM closeout matrix.

## Phases

Same A/B/C/D/E shape as Task 93/94.

## Per-AM validation gate

Per Task 93/94 structure, with the per-ISA stop condition
adjusted to 1.5× scoring-share (vs Task 93/94's 2×) because of
the per-candidate correction overhead.

## Stop conditions

- If FMA reordering on SIMD path exceeds the ADR-076 ULP
  tolerance for a particular AM × corpus cell: investigate
  whether the algebra requires strict accumulation order;
  document and decide whether to tighten to Option A on that
  AM or accept the recall variance with bench-level recall
  preservation as the gate.

## Coordination

- **Depends on Task 91 Phase 1** trait surface — specifically
  the `CandidateMeta::GammaAndResidualSigns { gamma, signs }`
  variant must land in the trait before Task 97 can register
  against it.
- **Depends on Task 92** infrastructure.
- **Parallel with Tasks 93–96, 98.**
- **Consumed by Task 99.**

## References

- Task 86 packet 002 (TurboVec QJL transferability)
- ADR-025 (quantization bit allocation MSE vs QJL)
- ADR-076 (universal block kernel pattern — Task 92)

## Estimated size

Medium. 4–6 weeks for one coder. Per-candidate side data
threading and FMA ULP-tolerance audit are the slowest pieces.
