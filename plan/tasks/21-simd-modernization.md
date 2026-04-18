# Task 21: SIMD Modernization — AVX-512 and ARM SVE

Status: proposed — two SIMD backends under existing runtime dispatch.

Executes ADR-039 (ARM SVE) and the AVX-512 investigation from the task-08
`Scoring Hot Path Investigation` note.

## Scope

Two modernization threads sharing one task because they ride the same
runtime-dispatch infrastructure and test the same equivalence harness:

1. **AVX-512 specializations** for x86_64 on top of the landed AVX2+FMA
   baseline. Target the three hottest kernels: FastScan LUT scoring,
   FWHT, and binary-sidecar POPCNT.
2. **ARM SVE/SVE2 backend** as a secondary aarch64 backend. Keep NEON
   as the baseline (Graviton 2 and older Apple Silicon); SVE/SVE2
   preferred where available (Graviton 3+, Apple M4+).

Both threads share: `src/quant/simd.rs` runtime dispatch, the
`TQVECTOR_SIMD` override, and the existing equivalence test coverage.

## Why pair them

- **Same dispatch layer.** Task 08 already built runtime feature
  detection; both extensions slot in as new `SimdBackend` variants.
- **Same equivalence contract.** Each new kernel must match the scalar
  path byte-for-byte on the existing test suite before it's eligible
  for dispatch.
- **Same risk profile.** Both are additive backends behind dispatch;
  failure means the old path runs. No broader blast radius.

## AVX-512 subtasks

- [ ] **Target detection.** Add `avx512f`, `avx512bw`, `avx512vbmi`,
  `avx512vpopcntdq` feature checks to the existing runtime dispatch.
  Gate each kernel on its specific required feature set (don't bundle
  into a single "avx512" flag).
- [ ] **FWHT AVX-512.** 512-bit butterfly halves the pass count for
  large FWHT sizes. Validate against scalar on production dims
  (`1536`, `2048`) and tail dims.
- [ ] **FastScan LUT AVX-512.** `_mm512_permutexvar_epi8` for 4-bit
  code-to-LUT gather. Pair with AVX-512 accumulator lanes. Measure
  vs AVX2 `_mm256_shuffle_epi8` path.
- [ ] **POPCNT AVX-512.** `_mm512_popcnt_epi8` for binary-sidecar
  scoring (ADR-031 path). On chips with `avx512vpopcntdq` this is
  a significant speedup over the scalar or AVX2 bit-manipulation
  variant.
- [ ] **Equivalence tests.** Extend the existing AVX2 equivalence suite
  to exercise the new kernels on production and tail dims.
- [ ] **Throughput benchmark.** Report AVX-512 vs AVX2 on the same
  Criterion cases task 08 used (`fwht/2048`, `prepare_ip_query/d1536_b4`,
  `score_ip_encoded/d1536_b4`) plus the FastScan LUT scorer cases.

## SVE / SVE2 subtasks

- [ ] **Target detection.** `sve` (ARMv8.2+), `sve2` (ARMv9+). Graviton
  3/4 expose SVE with 256-bit physical vectors; Apple M4 exposes
  SVE2 with 128-bit physical vectors. Code must be VLA
  (vector-length-agnostic) and not assume a specific vector bit-width.
- [ ] **FastScan LUT scorer (SVE2).** `tbl` instruction for code-to-LUT
  gather with scalable predication. VLA lets one binary run well on
  any SVE width.
- [ ] **FWHT (SVE).** Scalable butterfly. Predicate-mask the tail pass
  rather than padding.
- [ ] **POPCNT (SVE2).** `cnt` plus horizontal reduction. On chips
  without SVE2 but with SVE, fall back to NEON `vcntq_u8`.
- [ ] **Equivalence tests on real hardware.** NEON validation gap noted
  in task 08 still applies; SVE validation needs Graviton 3+ and/or
  Apple M4. CI access strategy: document which runner to use; keep
  scalar fallback authoritative until hardware runs green.
- [ ] **Throughput benchmark.** NEON vs SVE/SVE2 on Graviton 3 for
  the same Criterion cases.

## Shared subtasks

- [ ] **`TQVECTOR_SIMD` override matrix.** Document the valid
  override strings so operators can pin a backend for comparative
  measurement. Matrix: `scalar | avx2 | avx512 | neon | sve | sve2`.
- [ ] **Feature-detection snapshot.** Extend the existing
  `src/quant/simd.rs` dispatch metadata so `EXPLAIN (tqvector)` can
  surface which backend the scan used. Ties into task 19's EXPLAIN
  counter activation.

## Owns

- ADR-039 (ARM SVE)
- Scoring Hot Path Investigation follow-up from task 08

## Dependencies

- Task 08 SIMD runtime dispatch (merged on main).
- Nothing format-side. All kernels operate on the existing
  TurboQuant and PqFastScan byte layouts.

## Unblocks

- Graviton 3+ / Apple M4+ cost-efficiency story (SVE).
- Sapphire Rapids / Genoa / Turin throughput ceiling (AVX-512).
- Marginal but compounding wins on the scan-kernel hot path; matters
  more once parallel scan (task 18) spreads work across more cores
  and per-core kernel throughput becomes the inner bound.

## Out of scope

- AMX / MME (matrix extensions). Different programming model, different
  kernel shape. Separate ADR if ever.
- RISC-V vector extension. Not enough deployment yet to justify.
- GPU offload. Explicitly ruled out — execution model mismatch with
  Postgres scan callbacks, no cloud Postgres supports it.

## Notes

- **Runtime cost scaling is sub-linear.** AVX-512 on the LUT scorer
  typically yields 1.5–2x over AVX2, not 2x. The real win is
  POPCNT-heavy paths like binary prefilter scoring.
- **SVE width portability is a feature.** A single VLA binary runs on
  128-bit, 256-bit, and 512-bit SVE implementations. Don't write
  separate kernels per width.
- **Test matrix grows.** Each backend needs hardware coverage; CI
  already budgets for Graviton runs — this task adds SVE-capable
  runners. Document the runner requirements clearly so they aren't
  surprising in CI reviews.
- **Order within the task.** Recommend AVX-512 first (x86_64
  hardware is universally available in CI), then SVE (requires
  Graviton 3+ runners).
