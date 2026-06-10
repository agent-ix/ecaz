# Task 96: TurboQuant no-QJL 2-bit Block Kernel Family

Status: deferred (2026-06-09; Phase 0 surface-inventory stop condition in `reviews/task-96/001-surface-inventory-stop-condition/` found no current TurboQuant no-QJL 2-bit AM consumer; stop condition accepted by reviewer. Per project decision, TQ mode/bit-allocation questions — including whether to introduce a 2-bit surface — are deferred to the complete post-kernel index × quant × mode profile under Task 99. Do not resume Task 96 before that profile lands.)
Owner: coder (to be assigned). Phase III parallel.
Priority: 2 (denser-than-4-bit kernel; depends on 2-bit storage adoption)

## Why

TurboQuant no-QJL 2-bit is the half-density sibling of the
4-bit kernel Task 87 shipped. Per ADR-025 + TurboQuant
literature, 2-bit packing trades off recall for higher
arithmetic intensity and lower storage. Where 4-bit uses a
16-entry LUT per dimension, 2-bit uses a 4-entry LUT — denser
register packing, more candidates per kernel inner-loop
iteration.

Current 2-bit scoring (where present) goes through
`score_ip_from_parts` per-candidate. No batched kernel. Lifts
the LUT32 pattern from Task 87 with a smaller per-dim LUT.

## Scope

### In scope

1. **Scalar block kernel** at `src/quant/lut32_2bit/scalar.rs`
   (or `lut32/scalar_2bit.rs` if Task 92 skeleton structures
   them as variants of one kernel family — depends on Phase 1
   skeleton).
2. **NEON variant** using `vqtbl1q_u8` for 4-entry LUT lookup
   across 32 lanes.
3. **SVE variant** using vector-length-agnostic SVE `tbl` with
   4-entry LUT. Report as SVE-256 only when the measured runtime
   vector length is 256 bits.
4. **AVX2 variant** using `_mm256_shuffle_epi8` for 4-entry
   LUT lookup.
5. **`QuantCodec` registration** on every AM that exposes
   TurboQuant no-QJL 2-bit (audit IVF, SPIRE, HNSW for 2-bit
   storage adoption first).
6. **Per-(AM × ISA) measurement** on surfaces with 2-bit
   storage.
7. **Recall byte-equal** per ADR-076.
8. **Per-AM closeout matrix.**

### Out of scope

- AMs without 2-bit TurboQuant surfaces. If 2-bit is only on
  one AM (e.g., SPIRE), Task 96 covers that AM only.
- Storage format work for 2-bit adoption — separate concern.

## Acceptance criteria

1. `src/quant/lut32_2bit/` (or equivalent) module live with
   scalar + NEON + SVE + AVX2.
2. Each AM with 2-bit TurboQuant routes scoring through
   Task 91's selected `QuantCodec` batch method for batches ≥ 32.
3. Recall byte-equal at every cell.
4. ≥ 2× scoring-share per ISA per AM.
5. End-to-end no regression beyond noise.
6. `pg_test` surfaces for 2-bit-using AMs pass.
7. Safety docs.
8. Per-AM closeout matrix.

## Phases

### Phase 0 — Surface inventory

- Audit current AMs for real 2-bit TurboQuant scoring consumers.
- If no AM exposes such a surface, file the Stop Condition packet
  immediately and do not implement speculative kernels.
- If the Task 92 skeleton can parameterize by LUT entry count,
  decide whether this is a thin instantiation of the Task 87/94 LUT
  family rather than a standalone kernel family.

Phases A/B/C/D/E then follow the Task 93/94 shape only for the
surfaces found in Phase 0.

## Per-AM validation gate

Per Task 93/94 structure.

## Stop conditions

- If no AM in tree currently exposes a 2-bit TurboQuant scoring
  surface, file a Stop Condition packet noting the surface gap
  and defer Task 96 to a follow-up after 2-bit surface
  adoption. The kernel itself is small enough to land
  speculatively but isn't useful without a consumer.

## Coordination

- **Depends on Task 91** for AM trait registration shape.
- **Depends on Task 92** infrastructure.
- **Shares LUT-lookup kernel structure with Task 94 (grouped-PQ)
  and Task 87 (TQ-4-bit).** If the Task 92 skeleton parameterizes
  by LUT entry count, Task 96 may be a thin instantiation
  rather than a full kernel family. Phase 1 should investigate.
- **Consumed by Task 99.**

## References

- Task 87 (TQ-4-bit predecessor)
- Task 86 packet 001 (transferability matrix)
- ADR-025 (quantization bit allocation MSE vs QJL)
- ADR-076 (universal block kernel pattern — Task 92)

## Estimated size

Small-medium. 3–5 weeks for one coder, lower if the Task 92
skeleton lets the kernel parameterize by LUT entry count
(then Task 96 collapses to a Phase A/E shape with the SIMD
variants shared with Task 94).
