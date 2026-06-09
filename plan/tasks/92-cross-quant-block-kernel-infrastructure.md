# Task 92: Cross-Quant Block Kernel Infrastructure + ISA Gating

Status: complete (2026-06-09, closeout `reviews/task-92/017-block-kernel-infra-closeout/`)
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (foundation for Tasks 93–99)

## Why

Task 87 Phase 7 shipped one block kernel (`src/quant/lut32.rs` for
TurboQuant no-QJL 4-bit). Tasks 93–98 will ship six more quant
kernel families. Each kernel family ships scalar + ARM NEON + ARM
SVE + Intel AVX2 variants, registers per-quant + per-AM
counters, and reports a per-(AM × quant × ISA) closeout cell.

Without shared infrastructure, each per-quant task reinvents the
same plumbing: ISA detection, `target_feature` gating, counter
attribution, off-path scalar-baseline counter for the ≥2× gate,
and bench-suite cross-quant axis. That's six rounds of duplicated
infrastructure work, six chances to drift the conventions, and
no canonical reference for adding new quants later.

Task 92 lands the shared infrastructure once. Tasks 93–98 then
instantiate kernels against it without writing their own plumbing.

## Architectural decisions (to be locked by ADR-076)

| Decision | Value |
|---|---|
| Block width | 32 candidates, universal |
| Kernel dispatch | Task 91's universal `QuantCodec` batch method (`score_ip_batch` today; `score_batch` only if Task 91 renames/generalizes it) |
| ISA detection | runtime via `is_x86_feature_detected!` / `is_aarch64_feature_detected!` |
| Per-kernel module layout | `src/quant/<kernel>/{mod.rs, scalar.rs, neon.rs, sve.rs, avx2.rs}` |
| Recall policy | strict `to_bits()` equality on scalar reference; ULP tolerance ≤ 4 ULP or 1e-6 relative on SIMD variants; recall@k preservation at bench level is the binding gate |
| Width-based gating | `batch.len() >= 32` routes to block kernel; smaller batches use scalar tail path |

ADR-076 was accepted in this task and captures these as a normative
architectural decision.

## Scope

### In scope

1. **ADR-076: Universal Block Kernel Pattern.** Drafts and accepts
   the
   decisions above plus the rationale. Sites: 32-block width vs
   SVE / NEON / AVX2 register widths, runtime ISA detection
   for fleet flexibility, ULP tolerance for SIMD freedom, off-path
   counter as the canonical kernel-vs-scalar measurement.
2. **Per-(AM × quant) scoring-share counters.** Extends the Task 87
   per-AM counter surface from `surface ∈ {spire, ivf, hnsw,
   unknown}` to `(surface, quant_kind, isa)` tuple attribution.
   Counter fields: `flushes`, `candidates`, `elapsed_nanos`,
   `kernel_flushes`, `kernel_candidates`, `kernel_isa`.
3. **Off-path scoring-share counter.** Instruments the per-candidate
   scalar scorer call sites so kernel-on vs kernel-off comparison is
   directly measurable as scalar nanos vs kernel nanos on the same
   workload. This is the methodology gap Phase 7 hit; Tasks 93–98 need
   this to claim the ≥2× scoring-share gate honestly. Must NOT
   change the scalar scorer's call shape in a way that invalidates
   Task 87 Phase 6 reproducibility.
4. **Runtime ISA detection helper.** A small `src/quant/isa.rs`
   module that detects available ISA features once at startup,
   caches the result, and exposes `current_isa()` returning an
   `enum Isa { Scalar, Neon, Sve, Sve2, Avx2 }` per-kernel. The ARM
   production measurement target is AWS Graviton 4 (Neoverse V2,
   SVE2). Graviton 4 packets target the `Sve2` dispatch branch when
   available and report the measured runtime vector length verbatim
   when making width-specific claims, for example `sve2-128` for the
   current target host class. Inference from host class alone is not
   sufficient for width-specific claims. Each kernel module's `mod.rs`
   uses this to pick the function pointer at first call.
5. **Per-kernel module layout convention.** Each Phase III quant
   kernel lives under `src/quant/<kernel>/`:
   - `mod.rs`: public entry point (`score_<quant>_batch`), ISA
     dispatch, shape validation, counter increment, scalar-tail
     fallback.
   - `scalar.rs`: scalar reference implementation (bit-exact).
   - `neon.rs`: NEON variant, `#[cfg(target_arch = "aarch64")]`,
     dispatched via runtime `is_aarch64_feature_detected!("neon")`.
   - `sve.rs`: SVE variant, runtime detection +
     `is_aarch64_feature_detected!("sve")`. The implementation must
     be vector-length agnostic or explicitly gated on the measured
     vector length; runtime detection skips it on hosts without SVE.
   - `avx2.rs`: AVX2 variant, runtime detection +
     `is_x86_feature_detected!("avx2")`.
   Each ISA module exposes a single `pub(super) fn
   score_block32_<isa>(...)`. The scalar module also exposes
   `score_scalar_tail` for the <32 fallback path.
6. **Reference kernel skeleton.** A template (committed to
   `src/quant/_skeleton/` and removed before Phase 6 closeout, or
   kept as docs under `docs/skeleton-kernel.md`) showing the
   canonical structure. Phase 1 design walkthrough verifies the
   skeleton fits all 7 in-scope quants.
7. **Bench suite cross-quant axis.** Extends `ecaz bench suite`
   so a suite step can sweep a `quant=<name>` axis. Suite cells
   become (AM, corpus, quant, isa, kernel-on/off). The runner
   resolves which (AM × quant × isa) cells are valid (skips
   structurally absent cells; explicit "missing kernel" markers
   for Phase III tasks that haven't shipped yet).
8. **Documentation.** Update `reviews/README.md` if needed; add
   `docs/block-kernel-development.md` with the skeleton template
   reference, ULP-tolerance contract, and ISA gating examples.

### Out of scope

- Any actual quant block kernel beyond Task 87's already-landed
  `lut32`. Those are Tasks 93–98.
- Task 91 trait migration work. Task 91 owns trait surface; this
  task owns the kernel-side infrastructure that registers into
  that trait.
- AVX-512 variant. Block-width-32 + AVX-512 is a follow-up after
  Task 99 if measurement shows justification.
- Apple silicon (M-series) variant. Local bench host is Intel
  desktop per project memory; Graviton 4 covers the SVE case.
- AWS Graviton 4 benchmark runs for real quant kernels. Task 92 ships
  shared infrastructure plus safe fallback stubs; Tasks 93-98 own the
  Graviton 4 smoke/bench evidence when a real SVE2 backend is introduced.

## Acceptance criteria

1. ADR-076 accepted (PROPOSED → ACCEPTED in this task's closeout).
2. Per-(AM × quant × isa) counter surface live and queryable via
   the same `ec_task87_candidate_batch_scoring_*` SQL functions
   (renamed if appropriate, e.g.
   `ec_block_kernel_scoring_*`). Counters tracked in bench
   command output line format compatible with Tasks 93–98 parsing.
3. **Off-path counter validated against Task 87 LUT32**: with
   `ec_*.candidate_batch_scoring=off`, the scalar counter accumulates
   nanos at the per-candidate scorer; with `=on`, the kernel
   counter accumulates kernel nanos. Closeout records calibration
   against total scoring nanos; ≤1% drift is the target for large
   stable batches, with workload-specific tolerances documented for
   small-batch/HNSW cells where clock granularity and instrumentation
   overhead dominate. Task 92 closeout records the approved local SPIRE
   TurboQuant LUT32 calibration from packet 014; Graviton 4 runtime
   evidence is deferred to the first Task 93-98 packet that lands a real
   SVE2 backend.
4. ISA detection helper unit-tested: on each available host,
   `current_isa()` returns the highest available variant per
   kernel; falls back to scalar when none detected.
5. Per-kernel module layout convention applied to Task 87
   `lut32.rs` as a backfilled reference impl: `lut32/scalar.rs`
   (current code), `lut32/{neon,sve,avx2}.rs` stubbed with safe
   scalar fallbacks or compile-time-disabled modules, never reachable
   `unimplemented!` paths in normal dispatch. This proves the
   conversion shape without claiming any
   ISA win on the existing kernel; Tasks 93–98 then land real ISA
   variants for their respective kernels.
6. Reference skeleton template documented; Phase 1 design walks
   the 7 in-scope quants through it (TQ-4bit ✓ via Task 87 backfill,
   TQ-2bit, TQ-QJL, RaBitQ, grouped-PQ, Hamming, HNSW exact modes)
   and confirms each fits.
7. Bench suite `quant=` axis tested end-to-end with a dummy quant
   cell (using Task 87 LUT32 as the populated cell + at least one
   `missing_kernel` marker proving the runner handles the gap).

## Phases

### Phase 1 — ADR-076 + module layout convention (design only)

- Draft ADR-076 with the architectural decisions table.
- Walk the 7 in-scope quants through the skeleton template;
  verify each fits without redesign.
- Specify per-kernel module conventions, ISA dispatch shape,
  counter attribution, and ULP tolerance contract precisely.
- Reviewer approves before Phase 2.

### Phase 2 — Counter surface + off-path counter

- Extend Task 87 counter surface to `(am, quant_kind, isa)`
  attribution.
- Wire off-path counter at the per-candidate scalar scorer call
  sites. Audit the wiring against Task 87 Phase 6 baseline
  reproducibility (must not invalidate packet 015 evidence).
- Validate: with kernel-on for TQ-no-QJL-4-bit, kernel counter
  hits 100% on SPIRE/IVF cells where Task 87 packets 020/021/022
  showed 100% LUT32 routing. With kernel-off, scalar counter
  accumulates the equivalent nanos.

### Phase 3 — Runtime ISA detection helper + LUT32 backfill

- Land `src/quant/isa.rs` with runtime feature detection +
  cached `current_isa()` per kernel.
- Backfill Task 87's `lut32.rs` into the new module layout
  (`lut32/scalar.rs` + stubbed `lut32/{neon,sve,avx2}.rs`).
  Validates the convention end-to-end on already-shipped code.
- Proves runtime dispatch overhead is ≤1% of kernel time
  (measure on TQ-4-bit cells from Task 87).

### Phase 4 — Reference kernel skeleton + docs

- Commit `docs/block-kernel-development.md` with the canonical
  skeleton, ISA gating example, counter increment example, ULP
  tolerance test pattern.
- Reference the Task 87 `lut32` backfill as the worked example.

### Phase 5 — Bench suite cross-quant axis

- Extend `ecaz bench suite` to handle `quant=<name>` in suite
  step expansion.
- Add `missing_kernel` marker for cells where the kernel hasn't
  shipped yet.
- Test end-to-end with the TQ-4-bit cell populated + one missing
  marker.

### Phase 6 — Closeout

- ADR-076 status flip PROPOSED → ACCEPTED.
- Closeout packet documenting:
  - all acceptance criteria met;
  - off-path counter validation result from the Task 92 local SPIRE
    TurboQuant LUT32 calibration run;
  - LUT32 module-layout backfill behavioral parity;
  - bench suite cross-quant axis dry-run + sample run.
  - deferred Graviton 4 disposition: Tasks 93-98 run the AWS smoke or
    benchmark evidence when they introduce real SVE2 kernels. The first
    such packet must report `Isa::Sve2`, measured runtime vector length,
    and direct `(AM, quant, isa)` counter rows. Full AWS performance
    benches belong to the kernel rollout tasks, not this infrastructure
    closeout.
- Status flip Task 92 → `complete`.

## Per-AM validation gate

Not applicable — this task ships infrastructure, not kernels.
Behavioral parity gate applies only to the Task 87 `lut32`
backfill, which must remain bit-equal vs the pre-backfill code
(packet 015's recall + storage gates still pass).

## Stop conditions

- If the off-path counter cannot be wired without changing the
  per-candidate scalar scorer's call signature in a way that
  breaks Task 87 Phase 6 baseline reproducibility, pause and
  document. Don't risk Phase 6 evidence. File a Stop Condition
  packet and discuss methodology.
- If ADR-076 walkthrough discovers a quant that genuinely doesn't
  fit the skeleton template, pause Task 92 and reopen the
  skeleton design. Don't ship infrastructure that one of Tasks
  93–98 can't use.

## Coordination

- **Depends on Task 91** reaching Phase 2 (trait audit + IVF
  retouch). Task 92's batch dispatch shape uses the grown trait
  method name selected by Task 91.
- **Off-path counter overlap with Task 87 closeout:** Task 87
  packet 021 reviewer feedback flagged off-path counter as a
  pre-closeout requirement. If Task 87 closeout coder lands the
  off-path counter as a Task 87 cleanup slice, Task 92 inherits
  it and skips Phase 2 wiring (only extends to per-(quant, isa)
  attribution). Otherwise Task 92 Phase 2 lands it.
- **Required by Tasks 93–98.** All Phase III tasks need ADR-076,
  the counter surface, the ISA detection helper, the module
  convention, and the bench suite extension.
- **Required by Task 99** for the cross-(AM × quant × ISA)
  aggregate matrix.

## References

- Task 87 (`CandidateBatch` + `lut32` Phase 7 kernel)
- Task 91 (`QuantCodec` trait migration)
- `spec/adr/ADR-071-unified-quantizer-interface.md`
- `spec/adr/ADR-072-index-local-quantized-codec-adapters.md`
- ADR-076 (universal block kernel pattern — accepted, authored by this task)
- Task 86 packet 002 (TurboVec block-kernel transferability matrix)
- pgvectorscale: `access_method/scan.rs` resort_buffer pattern

## Estimated size

Medium-large. 4–6 weeks for one coder including ADR drafting,
counter surface extension, LUT32 backfill, bench suite extension,
and closeout. The counter wiring is the highest-risk slice
because it must not break Task 87 Phase 6 baseline reproducibility.
