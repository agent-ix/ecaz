# Task 103: Intel AVX2 Kernel Gap Closure (Pre-AWS Intel Lane)

Status: proposed (2026-06-10; in-epic per operator decision — created so
the AWS Graviton-vs-Intel price/performance comparison runs against a
complete Intel kernel matrix, not one with `missing_kernel` holes)
Owner: coder (assigned 2026-06-10: the Task 102 author). Phase III.
Priority: 1 (blocks the AWS Intel profile lane; ARM lanes unaffected)

## Why

The operator wants the Task 99 per-ISA comparison to answer a concrete
procurement question: **is there a price/performance tradeoff between
low-cost Graviton and Intel for this workload?** That comparison is only
honest if every quant family is final-shape on both ISAs. The ARM side is
final after Tasks 93–98 + 101 + 102. The Intel side still carries the
deferrals absorbed into Task 99 from the Tasks 93/95/98 closeouts — but
Task 99's own scope excludes new kernel work ("All kernels are Tasks
93–98"), and those tasks are complete. Task 103 owns the remaining Intel
work, following the Task 102 precedent (new in-epic task for kernel gaps
rather than reopening closeouts).

Intel AVX2 kernel matrix as of `2f99971c5`:

| Family | Intel AVX2 state |
| --- | --- |
| lut32 | real (Task 102, measured 4.5×) |
| qjl32 | real (Task 97) |
| grouped_pq_block | real (Task 94 F8) |
| rabitq32 | **landed but never validated/benched on Intel** (Task 93 deferral) |
| hamming32 | **scalar POPCNT only; AVX2 question open** (Task 95 deferral) |
| int8_approx32 | **NEON only; AVX2 missing** (Task 98 deferral, `vpmaddubsw` named) |
| tiled_lut32 | **scalar only; disposition open** (Task 98 deferral) |

## Scope

### In scope

1. **int8_approx32 AVX2 kernel**: `vpmaddubsw`-style integer path,
   integer-exact with the scalar reference (the family's established
   contract), compatible with the Task 101 width cascade and the Task 102
   octet/partial conventions. This is the highest-expected-value item: an
   integer fast-scan instruction path on the exact-mode lane whose width
   economics improved with octet-granular tails.
2. **tiled_lut32 disposition**: local A/B of tiled_lut vs post-Task-102
   full_lut on the standard fixtures first. If full_lut dominates,
   document retirement/deprioritization of the tiled lane (no SIMD built
   for a losing lane); if tiled retains a niche (e.g., larger dims), build
   the AVX2 variant for it. Decision recorded either way — an honest
   `structurally_absent`/retired cell beats an unbuilt `missing_kernel`.
3. **hamming32 AVX2-vs-POPCNT decision**: measure the current scalar
   hardware-popcount rate on Intel, prototype-or-estimate the AVX2
   (vpshufb nibble-popcount / Harley-Seal) alternative, and record a
   keep/skip decision with numbers. Expected return is bounded (NEON
   measured 1.10–1.17×); a documented skip is an acceptable outcome,
   mirroring Task 95's Graviton SVE scope-out.
4. **rabitq32 AVX2 validation**: compile/runtime parity (the
   when-available test suite) plus local bench cells on a rabitq fixture
   with counter attribution — the backend landed in Task 93 without Intel
   evidence.
5. **Local evidence per the established gates** for every cell this task
   changes: bit-exact/integer-exact parity per the family's anchor
   contract, recall byte-equal, direct counter rows under `isa=avx2`,
   width histograms, kernel-on/off end-to-end, release-backend provenance
   (suite preflight), all suite-driven (FR-038).

### Out of scope

- ARM/NEON/SVE work of any kind (final-shape since Task 102 packet 002;
  G4 evidence is the trip's job).
- AVX-512 (remains the deliberately deferred tier).
- Quantized-LUT lut32 variant (deferred indefinitely — Task 99 absorbed
  deferrals, 2026-06-10).
- Dispatch-layer changes (Task 101 owns the driver; this task only adds
  kernels behind existing entry points).
- The AWS runs themselves and the price/performance analysis (Task 99
  profile, both lanes).

## Acceptance criteria

1. int8_approx32 AVX2 kernel real, integer-exact, returning `Isa::Avx2`;
   scoring-share ≥2× target / 1.5× floor against the same-head scalar
   anchor on the standard fixtures, or a documented stop-condition.
2. tiled_lut32 disposition decided with A/B evidence: retired/deprioritized
   with rationale, or a real AVX2 kernel meeting the same gates as (1).
3. hamming32 keep/skip decision recorded with Intel measurements.
4. rabitq32 AVX2 parity + bench cells with `isa=avx2` counter rows.
5. Recall byte-equal at every measured cell; end-to-end no regression
   beyond noise.
6. After this task, the Intel column of the Task 99 matrix contains no
   `missing_kernel` cells — only real kernels or documented
   retired/skip/structurally-absent decisions.

## Sequencing

In-epic, **before the AWS Intel profile lane** (Task 99 item 9): Intel
kernels must be final-shape before paid Intel evidence is collected, for
the same single-trip economics as the G4 pin. Intel kernel work does not
touch ARM lanes, so G4 preparation proceeds in parallel.

## References

- Task 99 "Absorbed deferrals from Tasks 93/95/98 closeouts" (2026-06-10)
- Task 93 closeout (rabitq32 AVX2 backend, deferred validation)
- Task 95 closeout (hamming32; NEON 1.10–1.17× bound, SVE scope-out precedent)
- Task 98 closeout (tiled_lut32/int8_approx32; `vpmaddubsw`; width-distribution facts)
- Task 102 packets 001/002 (kernel shape precedents, octet/partial conventions,
  release-backend provenance discipline)
- ADR-076; `docs/block-kernel-development.md`
