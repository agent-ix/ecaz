# Task 95: Hamming / Binary Fingerprint Block Kernel Family

Status: complete (2026-06-10, closeout `reviews/task-95/003-closeout-matrix/`,
reviewer-approved with documented deferrals: Graviton SVE scoped out on
measured grounds — hardware popcount bounds same-algebra SIMD at 1.10-1.17x;
the AVX2-vs-POPCNT question is recorded into Task 99 as the Intel-lane
empirical item)
Owner: coder (to be assigned). Phase III parallel.
Priority: 2 (small task, shares popcount structure with Task 93)

## Why

Binary fingerprint / Hamming distance scoring is the
prefilter / approximate path on:

- DiskANN binary-sidecar prefilter (`DiskannPreparedPrefilter::
  BinarySidecar`) — Task 29a.
- Any IVF or HNSW binary-sidecar paths if present.

Hamming kernels are the simplest of the block-kernel family:
`popcount(query_words XOR code_words)`. SIMD popcount + xor is
4–8× over scalar. Block-kernel pattern amortizes query_words
load across 32 candidates.

This task is sized small because the algebra is trivial and
shares the popcount kernel structure with Task 93 (RaBitQ).
Worth landing for completeness of the matrix and for the
DiskANN binary-sidecar prefilter latency improvement.

## Scope

### In scope

1. **Scalar block kernel** at `src/quant/hamming32/scalar.rs`.
2. **NEON variant** using NEON `cnt` + `veor`.
3. **SVE variant** using SVE `cnt` + predication. Report as
   SVE-256 only when the measured runtime vector length is 256 bits.
4. **AVX2 variant** using `_mm256_xor_si256` plus an
   AVX2-compatible popcount strategy such as nibble-LUT/`pshufb` +
   `_mm256_sad_epu8`. VPOPCNTDQ is reserved for a future AVX-512
   variant.
5. **`QuantCodec` registration** in DiskANN binary-sidecar
   prefilter + any other binary scoring sites.
6. **Per-(AM × ISA) measurement** on DiskANN surfaces with
   binary sidecar enabled.
7. **Recall byte-equal** + scalar bit-equality + SIMD ULP
   tolerance per ADR-076.
8. **Per-AM closeout matrix.**

### Out of scope

- New binary fingerprint AM coverage where not currently
  present.
- Multi-bit Hamming variants (Jaccard, weighted Hamming) —
  follow-up if a project requirement appears.

## Acceptance criteria

1. `src/quant/hamming32/` module live with scalar + NEON +
   SVE + AVX2.
2. DiskANN binary-sidecar prefilter routes through
   Task 91's selected `QuantCodec` batch method for batches ≥ 32.
3. Recall byte-equal at every cell.
4. ≥ 2× scoring-share per ISA.
5. End-to-end no regression beyond noise.
6. `pg_test` surfaces for DiskANN binary-sidecar pass.
7. Safety docs on intrinsic-using modules.
8. Per-AM closeout matrix.

## Phases

Same A/B/C/D/E shape as Task 93/94. Phase A scalar; B NEON +
Graviton 4 forced-NEON; C SVE + Graviton 4; D AVX2 + Intel; E
closeout matrix.

## Per-AM validation gate

Per Task 93/94 structure: recall byte-equal, ≥ 2× scoring-share
per ISA, no end-to-end regression, storage unchanged, `pg_test`
passing.

## Stop conditions

Same as Task 93/94.

## Coordination

- **Depends on Task 91 Phase 5** (DiskANN onto `QuantCodec`).
- **Depends on Task 92** infrastructure.
- **Shares popcount kernel structure with Task 93.** If Task 93
  lands a shared popcount helper, Task 95 reuses it. If Task 95
  lands first, factor the helper now so Task 93 can adopt.
- **Consumed by Task 99.**

## References

- Task 29a (DiskANN binary sidecar prefilter)
- ADR-076 (universal block kernel pattern — Task 92)

## Estimated size

Small-medium. 3–5 weeks for one coder. Smaller than 93/94
because algebra is trivial; cloud measurement overhead still
dominates the schedule.
