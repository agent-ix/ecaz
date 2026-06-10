# Task 97 qjl32 Counter Kind and Design

## Scope

This packet resolves the approved Phase 0 follow-up before any qjl32 kernel
records a row:

- add `QuantCodecKind::TurboQuantQjl` with label `turboquant_qjl`
- keep the existing `turboquant` label unchanged for no-QJL lut32
- design `src/quant/qjl32/{mod,scalar,neon,sve,avx2}.rs`
- target IVF, SPIRE, and HNSW current QJL/gamma-aware paths only
- keep DiskANN and standard 1536d/4-bit cells out of Task 97

No qjl32 module is implemented in this packet.

## Counter Attribution

Task 97 qjl32 rows must use:

```text
BlockKernelCounterKey {
    surface: <ivf|spire|hnsw>,
    quant_kind: QuantCodecKind::TurboQuantQjl,
    isa: <backend-returned isa>,
}
```

Scalar tails route through:

```text
record_block_scalar_score_for(surface, QuantCodecKind::TurboQuantQjl, ...)
```

This keeps direct `[block-kernel-counters]` rows separable:

```text
quant=turboquant      # Task 87 / lut32 no-QJL 4-bit
quant=turboquant_qjl  # Task 97 / qjl32 gamma + signs
```

The Task 87 compatibility `lut32_*` fields remain keyed only by
`QuantCodecKind::TurboQuant`, so qjl32 rows cannot be reported as lut32
kernel rows.

## Module Shape

`src/quant/qjl32/` follows ADR-076:

```text
src/quant/qjl32/
  mod.rs
  scalar.rs
  neon.rs
  sve.rs
  avx2.rs
```

Responsibilities:

- `mod.rs`: shape validation, 32-wide block/tail routing, runtime dispatch,
  and backend-returned ISA.
- `scalar.rs`: bit-exact reference and scalar tail scorer.
- `neon.rs`: NEON backend or scalar fallback returning `Isa::Scalar`.
- `sve.rs`: SVE/SVE2 backend or scalar fallback returning `Isa::Scalar`.
- `avx2.rs`: AVX2/FMA backend or scalar fallback returning `Isa::Scalar`.

The qjl32 family stays separate from lut32. The algorithms differ in row
width, bit decoding, sign-side accumulation, and per-candidate gamma combine.
Shared low-level helpers are allowed only when byte-identical and extracted as
small helpers; kernel entry points are not unified.

## Entry Point

Proposed public module entry:

```rust
pub(crate) fn score_turboquant_qjl_block32(
    quantizer: &ProdQuantizer,
    prepared: &PreparedQuery,
    codes: [&[u8]; BLOCK_WIDTH],
    gammas: [f32; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa
```

`codes` are the current production payload bytes: MSE-packed bytes followed by
QJL sign bytes. `gammas` come from `CandidateMeta::Gamma`. Task 97 does not
split residual signs into `CandidateMeta::GammaAndResidualSigns`.

## Shape Validation

`mod.rs` validates before scoring and before counters increment:

- `out_scores.len() == batch.len()`
- `quantizer.exact_score_mode() == ExactScoreMode::MseLutQjl`
- `quantizer.bits == 4` for the initial production surface
- `quantizer.original_dim != 1536`
- each payload has exactly `mse_code_len(dim, 4) + qjl_code_len(dim)` bytes
- each payload metadata is `CandidateMeta::Gamma(gamma)`

The scalar implementation may accept the same split payload slices internally:

```text
mse_packed = &code[..mse_len]
qjl_packed = &code[mse_len..mse_len + qjl_len]
```

## Scalar Pseudocode

The scalar reference must match the existing pre-kernel scorer with
`f32::to_bits()` equality:

```text
for candidate lane:
    mse_sum = 0.0f32
    qjl_sum = 0.0f32
    for dim in 0..original_dim:
        idx = mse_index_at(mse_packed, dim, 3)
        mse_sum += codebook[idx] * prepared.rotated[dim]
        sign = if qjl_sign_at(qjl_packed, dim) { 1.0 } else { -1.0 }
        qjl_sum += prepared.sq[dim] * sign
    out[lane] = mse_sum + gamma[lane] * prepared.qjl_scale * qjl_sum
```

For canonical `bits=4` QJL, `mse_bits(dim, bits) == 3`, so qjl32 starts with
the 3-bit MSE path only. Other bit widths stay out of scope until Task 99
decides whether to create those production surfaces.

## Block Pseudocode

The scalar block implementation keeps candidate order and the existing
per-candidate accumulation order:

```text
sums_mse[32] = 0
sums_qjl[32] = 0
for dim in 0..original_dim:
    for lane in 0..32:
        idx = read_3bit_index(codes[lane].mse, dim)
        sums_mse[lane] += codebook[idx] * prepared.rotated[dim]
        sums_qjl[lane] += prepared.sq[dim] * sign(codes[lane].qjl, dim)
for lane in 0..32:
    out[lane] = sums_mse[lane] + gammas[lane] * prepared.qjl_scale * sums_qjl[lane]
```

SIMD backends may reorder within the ADR-076 floating-point tolerance, but the
scalar reference is strict and remains the parity oracle.

## SIMD Strategy

### AVX2/FMA

- Process 8 candidate lanes at a time.
- For each dimension or small dimension group, decode 3-bit MSE indices for 8
  candidates.
- Use an 8-entry codebook vector with `_mm256_permutevar8x32_ps` where
  practical, mirroring the existing per-candidate AVX2 3-bit decode strategy.
- Expand QJL sign bits to `+1.0/-1.0` lanes and accumulate `sq * sign`.
- Keep separate MSE and QJL accumulators, then combine with
  `gamma * qjl_scale` per lane.
- Tolerance: ADR-076 `<= 4 ULP` or `1e-6` relative, whichever is larger.

### NEON

- Process 4 candidate lanes at a time.
- Use the existing NEON 3-bit decode pattern as the first implementation
  anchor.
- Expand sign lanes to `float32x4_t` and accumulate QJL terms separately.
- Combine with `vfmaq_f32` where available.
- Tolerance: ADR-076.

### SVE/SVE2

- Use vector-length-agnostic predicated loops across candidate lanes.
- Prefer SVE2 table/decode helpers only when runtime feature detection reports
  `sve2`; fallback to scalar and return `Isa::Scalar` otherwise.
- Graviton 4 packets must report `Isa::Sve2` and the measured runtime vector
  length verbatim before making width-specific claims.
- Tolerance: ADR-076.

## QuantCodec Integration

Add a helper parallel to the no-QJL path:

```text
score_turboquant_qjl_batch_for(surface, quantizer, prepared, batch, out)
```

AM routing:

- IVF: `IvfPreparedQuery::TurboQuant(prepared)` and
  `IvfQuantizerProfile::TurboQuant` route to qjl32 when
  `quantizer.exact_score_mode() == MseLutQjl`.
- SPIRE: `SpirePreparedAssignmentScorer::TurboQuant` routes to qjl32 when
  `no_qjl_4bit_lut.is_none()` and candidate batch scoring is enabled.
- HNSW: `HnswTurboQuantPreparedQuery::Exact(prepared)` routes to qjl32 when
  the cached quantizer is QJL-active. Non-default full/tiled/int8 exact modes
  remain no-QJL Task 98 surfaces.

All three AMs must continue to accept scalar fallback for `batch.len() < 32`
and record tails under `(surface, TurboQuantQjl, Isa::Scalar)`.

## Tests

Minimum local tests before implementation proceeds past scalar:

- counter-kind test proves `turboquant_qjl` direct rows exist and qjl rows do
  not increment `lut32_*` compatibility fields
- `dim=1024,bits=4` reachability asserts `ExactScoreMode::MseLutQjl`
- `len < 32`, `len == 32`, and `len > 32` scalar parity against
  `score_ip_from_parts`
- shape mismatch rejects before counters increment
- IVF/SPIRE/HNSW QuantCodec paths preserve candidate order and score parity

SIMD tests add backend-specific tolerance checks when each backend lands.

## Measurement Plan

Local benchmark evidence uses `ecaz bench suite` only:

- synthetic corpus, `dim=1024,bits=4,seed=42`
- one replicated table per AM
- AMs: IVF, SPIRE, HNSW
- packet-local SuiteConfig and packet-local logs
- direct `[block-kernel-counters]` rows with `quant=turboquant_qjl`
- standard 1536d rows marked no-QJL/absent for Task 97

No CI, AWS smoke, or AWS benchmark is part of this packet. Graviton 4 evidence
waits for the approved SVE2 lane.
