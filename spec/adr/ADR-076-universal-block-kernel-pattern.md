---
type: ADR
id: ADR-076
title: "Universal Block Kernel Pattern"
status: ACCEPTED
impact: Affects Task 91 QuantCodec migration, Task 92 kernel infrastructure, Tasks 93-99 quant kernel rollout, FR-038 bench suites, and Task 87 scoring counters.
date: 2026-06-09
---
# ADR-076: Universal Block Kernel Pattern

## Context

Task 87 shipped the first candidate-batched block scorer for TurboQuant
no-QJL 4-bit through `src/quant/lut32.rs`. Tasks 93-98 will add more
quantized scoring kernels across TurboQuant, RaBitQ, grouped-PQ, binary,
and HNSW exact-score modes.

Without one kernel pattern, each quant task would choose its own block
width, runtime ISA detection, counter attribution, scalar fallback, and
SIMD tolerance policy. That would make cross-quant benchmark matrices hard
to compare and would make AM integrations depend on per-quant special
cases.

Task 91 owns the common `QuantCodec` scoring interface. Task 92 owns the
kernel-side infrastructure that registers into that interface.

## Decision

Adopt a universal block kernel pattern for all compressed-domain batch
scorers.

Task 92 accepted this ADR after landing the shared counter surface, runtime ISA
helper, LUT32 module-layout backfill, block-kernel development documentation,
bench-suite quant axis, and off-path scalar counter calibration methodology.

### Dispatch Entry Point

All AM scan loops call the Task 91-selected `QuantCodec` batch method.
The method name remains `score_ip_batch` while the project only exposes
inner-product quantized scan scoring. If non-IP quantized metrics become
real scan paths later, add a metric-specific sibling instead of renaming
the current method during Task 92.

Each `QuantCodec` implementation owns its per-quant dispatch decision:
shape validation, width gating, runtime ISA choice, scalar tail routing,
and counter attribution happen under that method. AM code should not call
ISA-specific kernel functions directly.

### Block Width

Use a universal block width of 32 candidates.

The dispatch contract is:

- `batch.len() >= 32`: use the kernel module for the largest whole
  block range, then score any tail through the scalar reference path.
- `batch.len() < 32`: use the scalar path directly.
- Shape mismatches fail before scoring and before counter increments.

The 32-candidate block is the common unit for AVX2 byte shuffle style
kernels, NEON table kernels, SVE vector-length agnostic loops, grouped-PQ
block layouts, and the already-landed LUT32 scorer.

### Module Layout

Every kernel family lives under:

```text
src/quant/<kernel>/
  mod.rs
  scalar.rs
  neon.rs
  sve.rs
  avx2.rs
```

Module responsibilities:

- `mod.rs`: public batch entry point, shape validation, width gating,
  runtime dispatch, counter attribution, and scalar tail fallback.
- `scalar.rs`: bit-exact scalar reference implementation plus
  `score_scalar_tail`.
- `neon.rs`: NEON implementation behind `#[cfg(target_arch = "aarch64")]`.
- `sve.rs`: SVE/SVE2 implementation behind `#[cfg(target_arch = "aarch64")]`.
  The implementation must distinguish `sve` and `sve2` runtime features,
  stay vector-length agnostic, or explicitly skip itself unless the measured
  vector length is supported.
- `avx2.rs`: AVX2 implementation behind `#[cfg(any(target_arch = "x86",
  target_arch = "x86_64"))]`.

Each ISA module exposes one block entry point named
`score_block32_<isa>`. Compile-time-disabled modules must compile to safe
scalar fallbacks or be absent from dispatch. Normal dispatch must never
reach `unimplemented!`.

### Runtime ISA Detection

Task 92 adds `src/quant/isa.rs` with:

```rust
pub(crate) enum Isa {
    Scalar,
    Neon,
    Sve,
    Sve2,
    Avx2,
}
```

Kernel modules use `is_x86_feature_detected!` and
`is_aarch64_feature_detected!` to select the highest valid implementation
for that kernel, cache the selected function pointer on first use, and
fall back to scalar when the host lacks the required features.

The ARM production measurement target is AWS Graviton 4 (Neoverse V2, SVE2).
Graviton 4 packets must target the `sve2` dispatch branch when that feature is
available and must report the measured runtime vector length verbatim when
making width-specific claims. Inference from host class alone is forbidden:
packets must report the measured runtime vector length verbatim, for example
`sve2-128` or `sve-256`.
If vector length cannot be measured, report only `sve` or `sve2` and do not
publish width-specific claims.

### Counter Attribution

Task 92 extends Task 87's scoring counters from AM-only attribution to
`(am, quant_kind, isa)`.

Counters must distinguish:

- total batch flushes, candidates, and elapsed nanos;
- kernel flushes, candidates, elapsed nanos, and selected ISA;
- off-path scalar flushes, candidates, and elapsed nanos when kernel
  routing is disabled or width gating sends candidates to scalar.

The off-path scalar counter is the canonical comparison for later
`>= 2x` scoring-share claims. It must not change the scalar scorer's
call shape in a way that invalidates Task 87's reproducibility evidence.

### Correctness Contract

Scalar reference implementations are strict:

- scalar output must match the pre-kernel implementation with
  `f32::to_bits()` equality for deterministic quant cells.
- integer/Hamming style kernels must match exact integer counts before
  conversion to score polarity.

SIMD variants may differ from scalar by at most 4 ULP or `1e-6` relative
error for floating-point accumulators, whichever is larger for the value
being compared. Bench-level recall@k preservation remains the binding gate
for accepting a SIMD variant.

## Consequences

- AM scan loops stay insulated from ISA details. They route through
  `QuantCodec::score_ip_batch` and receive scores in candidate order.
- The Task 87 LUT32 scorer must be backfilled into this layout before
  Task 92 closes.
- Tasks 93-98 can implement quant-specific math without redefining
  counters, dispatch, width gating, or tolerance rules.
- SVE/SVE2 kernels must be portable across vector lengths. Graviton 4 is the
  target ARM server host and uses the SVE2 dispatch branch when available, but
  the code must not assume a hard-coded vector length unless dispatch validates
  it and packet evidence reports the measured width.
- AVX-512 and Apple silicon variants are explicit follow-ups, not part of
  the Task 92 kernel infrastructure.

## Alternatives Considered

### Per-Quant Dispatch Conventions

Rejected. It would let each quant task drift on width gates, counters, and
fallback behavior, making Task 99's aggregate matrix less trustworthy.

### Compile-Time ISA Selection Only

Rejected. The same binary needs to run correctly across local Intel hosts,
AWS Intel hosts, and AWS Graviton 4 hosts. Runtime detection keeps one
portable binary with host-appropriate dispatch.

### Rename `score_ip_batch` to `score_batch` in Task 92

Rejected for now. The live compressed-domain scan contract is inner
product. A rename would add churn before non-IP quantized scoring exists.
Task 91 may still revisit this if its trait audit finds a concrete
multi-metric requirement.

### Fixed-Width SVE/SVE2 Kernels

Rejected as the default. Graviton 4 is the measurement target, but the ARM
contract is vector-length agnostic. A fixed-width SVE or SVE2 variant may be
added only when dispatch checks the runtime vector length and packets report
the measured ISA-width label, such as `sve2-128`.

## Related Decisions

- ADR-071: Unified quantizer interface across access methods.
- ADR-072: Index-local quantized codec adapters.
- Task 87: Candidate batching and first LUT32 kernel.
- Task 91: Cross-AM `QuantCodec` migration.
- Task 92: Cross-quant block kernel infrastructure and ISA gating.
