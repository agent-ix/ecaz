# Review Request: NEON 3-Bit Scoring Specialization

Commits:
- `0088255` Fix NEON QJL sign lane offset within byte
- `6ec3fa0` Specialize NEON 3-bit scoring with vectorized index decode

Scope:
- `src/quant/prod.rs`

## Bug Fix (0088255)

The NEON scoring loop steps by 4 dims per iteration, but `vld1q_f32(sign_lanes.as_ptr())`
always loaded the first 4 of the 8-element `qjl_sign_lanes` array. For dims 4-7 within each
QJL byte, this read the wrong sign bits (bits 0-3 instead of 4-7), corrupting the QJL
correction term for half of all dimensions.

Fix: `vld1q_f32(sign_lanes.as_ptr().add(dim_index % 8))`

The AVX2 path is unaffected (loads all 8 lanes with `_mm256_loadu_ps`, advances by 8 or 32).
The scalar path is unaffected (uses per-element `qjl_sign_at`).

## NEON Specialization (6ec3fa0)

### `score_ip_from_split_parts_neon` — rewritten for 3-bit path

Old (4-lane, scalar index decode):
```
for lane in 0..4:
    centroid_index = mse_index_at(mse_packed, dim + lane, 3)
    mse_values[lane] = codebook[centroid_index] * rotated[dim + lane]
```

New (8-lane, NEON index decode):
```
word = decode_eight_3bit_aligned_word(mse_packed, dim)
broadcast = vdupq_n_u32(word)
idx_lo = vandq_u32(vshlq_u32(broadcast, [0,-3,-6,-9]), 0x7)
idx_hi = vandq_u32(vshlq_u32(broadcast, [-12,-15,-18,-21]), 0x7)
// scatter-load codebook (scalar, no NEON gather)
mse_acc0 = vfmaq_f32(mse_acc0, cb_lo_vec, rotated_lo_vec)
mse_acc1 = vfmaq_f32(mse_acc1, cb_hi_vec, rotated_hi_vec)
```

Key changes:
- 8 dims per iteration (aligned with packed byte boundaries)
- NEON variable shift (`vshlq_u32` with negative values) for parallel index decode
- `vfmaq_f32` for fused multiply-accumulate
- 2-accumulator chain (`mse_acc0/1`, `qjl_acc0/1`) for instruction-level parallelism
- Non-3-bit path retains existing 4-lane scalar-index behavior

### `score_ip_mse_codes_neon` — new

Same pattern for code-to-code scoring. Wired into `SimdBackend::Neon` dispatch.

### Limitation: no NEON gather

NEON lacks AVX2's `_mm256_permutevar8x32_ps` for codebook lookup. The workaround is:
1. NEON decode indices → store to temp array
2. Scalar codebook loads from the temp indices
3. Load codebook values back into NEON register

This means the codebook gather is still scalar. The vectorization benefit comes from:
- Parallel index decode (1 broadcast + 2 shifts + 2 masks vs 8× `mse_index_at`)
- FMA accumulation (vs separate multiply + scalar sum)
- 2-accumulator ILP

### Testing

- Cross-compiled for `aarch64-unknown-linux-gnu` — compiles clean
- All 236 x86_64 tests pass (NEON code is `#[cfg(target_arch = "aarch64")]`)
- Requires aarch64 hardware for runtime testing and benchmarking

Please review:
- whether the scatter-load codebook pattern is worth the extra code vs keeping the scalar
  approach with just the bugfix
- whether 2 accumulators is sufficient for NEON's 4-lane width, or whether we should stay
  with 1 accumulator since the loop body is smaller than AVX2's
- whether the non-3-bit NEON path should also get FMA accumulation
