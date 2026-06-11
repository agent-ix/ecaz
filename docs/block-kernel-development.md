# Block Kernel Development

Task 92 and ADR-076 define one pattern for compressed-domain block scoring
kernels. Use this document when adding kernel families for Tasks 93-98.

## Scope

Block kernels score batches of 32 candidates through the relevant
`QuantCodec::score_ip_batch` implementation. AM scan code should not call
ISA-specific kernel functions directly. The quant codec owns:

- shape validation;
- width gating;
- runtime ISA dispatch;
- partial-width and scalar remainder handling;
- counter attribution.

The reference implementation is `src/quant/lut32/`, the Task 87 TurboQuant
no-QJL 4-bit scorer backfilled into the Task 92 module layout.

## Module Layout

Every kernel family uses this layout:

```text
src/quant/<kernel>/
  mod.rs
  scalar.rs
  neon.rs
  sve.rs
  avx2.rs
```

Responsibilities:

- `mod.rs`: public entry point, shape validation, block-width gating, runtime
  dispatch, partial-width or scalar remainder routing, and counter-facing
  return values.
- `scalar.rs`: bit-exact reference implementation plus scalar remainder
  scoring.
- `neon.rs`: aarch64 NEON backend or a safe scalar fallback.
- `sve.rs`: aarch64 SVE/SVE2 backend or a safe scalar fallback.
- `avx2.rs`: x86/x86_64 AVX2 backend or a safe scalar fallback.

Normal dispatch must never reach `unimplemented!`. If a backend file exists
only as a fallback stub, it must call the scalar implementation and return
`Isa::Scalar` for counter attribution.

## Entry Point Shape

`mod.rs` exposes a block function that returns the ISA actually used by the
backend:

```rust
pub(crate) fn score_<kernel>_block32(
    prepared: &PreparedKernelState,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa {
    let selected = crate::quant::isa::current_isa();
    match selected {
        Isa::Avx2 => avx2::score_block32_avx2(prepared, &codes, out_scores),
        Isa::Sve2 | Isa::Sve => sve::score_block32_sve(prepared, &codes, out_scores),
        Isa::Neon => neon::score_block32_neon(prepared, &codes, out_scores),
        Isa::Scalar => scalar::score_block32_scalar(prepared, &codes, out_scores),
    }
}
```

The backend return value is the attribution value, not merely the detected host
capability. A fallback SVE file that delegates to scalar returns `Isa::Scalar`.
A real Graviton 4 SVE2 backend returns `Isa::Sve2`.

## Width Gating

Use a universal base block width of 32 candidates.

- shape mismatches fail before scoring, before output mutation, and before
  counter increments;
- `batch.len() >= 32`: score the largest whole block32 range with the selected
  backend;
- supported remainders use the family-specific partial-width cascade
  (`partial`, `octet`, or equivalent) before scalar fallback;
- unsupported widths, disabled kernels, and missing runtime features use the
  scalar reference path.

Scalar remainders must be recorded separately under `Isa::Scalar`, even when
whole or partial blocks in the same batch use `Isa::Sve2`, `Isa::Sve`,
`Isa::Neon`, or `Isa::Avx2`.

## Counter Attribution

Kernel call sites record rows by `(surface, quant_kind, isa)`.

For every successful batch:

- whole blocks increment `kernel_*` fields on the backend-returned ISA row;
- supported partial-width blocks increment the appropriate kernel row and
  width bucket for the backend-returned ISA;
- scalar remainders increment `scalar_*` fields on the `Isa::Scalar` row;
- fallback backend stubs that delegate to scalar return `Isa::Scalar`, so they
  do not publish false ISA-specific rows.

The current LUT32 worked example is:

```rust
let isa = crate::quant::lut32::score_lut_no_qjl_4bit_block32(
    &prepared.lut,
    quantizer.original_dim,
    mse_codes,
    out_scores,
);
timing.kernel_isa = Some(isa);
```

The caller later builds:

```rust
BlockKernelCounterKey {
    surface,
    quant_kind: QuantCodecKind::TurboQuant,
    isa: timing.kernel_isa.unwrap_or(Isa::Scalar),
}
```

Do not derive the counter ISA by calling `current_isa()` again at the AM call
site. Use the backend-returned value so fallback stubs and future per-kernel
gates stay honest.

## Correctness Tests

Scalar reference tests are strict. Where the previous scorer is deterministic,
assert `f32::to_bits()` equality against the pre-kernel scalar path.

SIMD variants may use ADR-076 tolerance:

- at most 4 ULP; or
- `1e-6` relative error, whichever is larger.

Bench-level recall@k preservation remains the binding acceptance gate for a
SIMD backend. Integer and Hamming kernels must match exact integer counts
before score polarity conversion.

Minimum test coverage for a kernel family:

- `len < 32` follows the family's supported partial-width or scalar remainder
  route and matches the scalar reference;
- `len == 32` scores one whole block and matches the reference within the
  required tolerance;
- `len > 32` scores whole blocks plus partial/scalar remainders and preserves
  candidate order;
- shape mismatch rejects before counters increment and before output mutation;
- counter rows report the backend-returned ISA for whole and partial blocks,
  width buckets for exact block32/partial/octet/scalar remainder, and
  `Isa::Scalar` for scalar remainders.

## Graviton 4 / SVE2 Evidence

AWS Graviton 4 is the ARM production measurement target. Packets must treat it
as Neoverse V2 with SVE2 available and must not use Graviton 3 assumptions.

For Graviton 4 packets:

- target the `Isa::Sve2` branch when runtime detection reports SVE2;
- report the measured runtime vector length verbatim when making width-specific
  claims, for example `sve2-128`;
- avoid width-specific claims if vector length was not measured;
- include counter evidence showing kernel rows under `isa=sve2` once a real
  SVE2 backend lands, with scalar remainders still under `isa=scalar`.

Task 92's infrastructure closeout does not require AWS Graviton 4 benchmarks:
it ships shared dispatch, counters, suite axes, docs, and safe fallback stubs.
The first Task 93-98 packet that introduces a real SVE2 backend owns the
Graviton 4 smoke evidence (`Isa::Sve2`, measured runtime vector length, direct
counter rows). Full Graviton 4 performance benches belong to the kernel rollout
task that makes the performance claim.

## Packet Evidence

Kernel implementation packets should include packet-local artifacts for:

- focused unit tests for scalar/reference parity and SIMD tolerance;
- counter tests or SQL/CLI smoke output showing `(surface, quant_kind, isa)`
  rows;
- benchmark-suite evidence for valid, missing-kernel, and structurally absent
  cells when the packet touches suite expansion;
- the exact host and ISA evidence used for Graviton 4 claims.

Do not cite terminal scrollback or temporary paths as durable evidence. Store
logs under the owning review packet's `artifacts/` directory.
