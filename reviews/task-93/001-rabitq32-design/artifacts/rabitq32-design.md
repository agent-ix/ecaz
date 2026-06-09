# Task 93 Phase 1 Design: RaBitQ 32-Candidate Block Kernel

## Scope

This packet covers the design for `src/quant/rabitq32/` only. It does not
change code. The implementation target is the primary deployed RaBitQ 1-bit
search-code path across IVF, DiskANN, HNSW, and SPIRE. Multi-bit RaBitQ remains
out of the first implementation slice unless Phase 2 proves the same entry
shape can preserve the existing scalar scorer without broadening the API.

Binding references:

- `plan/tasks/93-rabitq-block-kernel-family.md`
- `spec/adr/ADR-076-universal-block-kernel-pattern.md`
- `docs/block-kernel-development.md`
- `src/quant/lut32/{mod,scalar,neon,sve,avx2}.rs`
- `src/am/common/{quant_codec,candidate_batch}.rs`
- `reviews/task-91/018-adr071-072-acceptance/feedback/2026-06-09-02-reviewer.md`

## Existing Scalar Contract

The scalar reference is the current RaBitQ scan scorer, not a newly invented
formula:

- IVF: `IvfQuantizer::score_ip_bits1_batch_from_payloads`, which delegates to
  `PreparedEstimator::estimate_ip_bits1_batch`.
- DiskANN: `DiskannRaBitQPrefilterCodec::score_ip_candidate`, which delegates
  to `PreparedEstimator::estimate_ip_scalar_only`.
- HNSW: `HnswRaBitQScanCodec::score_ip_candidate`, which delegates through
  `RaBitQScorer::score` to the same scalar estimator.
- SPIRE assignment: `SpireAssignmentScorer::RaBitQ` uses
  `PreparedEstimator::estimate_ip_scalar_only`.

For bits=1, the current scorer computes:

1. `sum_q_dequant` from packed candidate signs and the prepared query state.
   The existing scalar fast path uses `bits1_byte_lut` so each packed byte maps
   to eight dequantized sign values, then multiplies those values by
   `query_rotated[i]`.
2. Per-candidate scalar metadata from the trailing RaBitQ fields:
   `candidate_norm`, `candidate_o_dot`, and `candidate_x_norm`.
3. The final estimate:

   ```text
   if abs(candidate_o_dot) < 1e-6
      or candidate_o_dot is not finite
      or candidate_x_norm <= 0
      or candidate_x_norm is not finite:
       score = 0.0
   else:
       score = candidate_norm * sum_q_dequant / (candidate_o_dot * candidate_x_norm)
   ```

Task 93's "popcount prototype" must preserve this scalar contract. The first
kernel will make the bitwise sign comparison/counting part block-oriented, but
the per-candidate correction remains a lane-local finish step using the scalar
metadata in each code.

## Module Shape

Implement the ADR-076 layout:

```text
src/quant/rabitq32/
  mod.rs
  scalar.rs
  neon.rs
  sve.rs
  avx2.rs
```

Planned public entry points:

```rust
pub(crate) const BLOCK_WIDTH: usize = 32;

pub(crate) struct RaBitQ32Prepared<'a> {
    pub(crate) dimensions: usize,
    pub(crate) code_len: usize,
    pub(crate) query_rotated: &'a [f32],
    pub(crate) dequant_lut: &'a [f32; 256],
    pub(crate) bits1_byte_lut: &'a [[f32; 8]; 256],
}

pub(crate) fn score_rabitq_bits1_block32(
    prepared: RaBitQ32Prepared<'_>,
    codes: [&[u8]; BLOCK_WIDTH],
    out_scores: &mut [f32],
) -> Isa;

pub(crate) fn score_rabitq_bits1_scalar(
    prepared: RaBitQ32Prepared<'_>,
    code: &[u8],
) -> f32;
```

`RaBitQ32Prepared` should be built from the existing `PreparedEstimator` /
`RaBitQScorer` state through narrow accessor methods added in Phase 2. The
accessors are intentionally read-only and avoid copying query state.

## Shape Validation

`mod.rs` validates before scoring and before counters increment:

- `out_scores.len() == batch.len()`;
- `bits == 1`;
- `code_len == code_len_for(dimensions, 1)`;
- every payload is at least `code_len`;
- every payload has `CandidateMeta::RaBitQ` or metadata that maps to zero gamma
  on existing call sites;
- `bits1_byte_lut` is present.

Shape mismatch returns `Err(String)` from the owning `QuantCodec::score_ip_batch`
implementation and records no counters.

## Width Gating and Tails

The batch path mirrors LUT32:

```text
if batch.len() < 32:
    scalar_start = now
    for candidate in batch:
        score_rabitq_bits1_scalar(...)
    record_block_scalar_score_for(surface, RaBitQ, batch.len(), elapsed)
else:
    block_start = 0
    while block_start + 32 <= batch.len():
        collect [&[u8]; 32]
        isa = score_rabitq_bits1_block32(prepared, codes, out_scores[block])
        timing.kernel_isa = isa
        timing.kernel_candidates += 32
        timing.kernel_elapsed += elapsed
        block_start += 32
    if block_start < batch.len():
        scalar-tail the remainder
        record tail under (surface, RaBitQ, Isa::Scalar)
```

The `score_block32_<isa>` return value is the counter attribution. Fallback
stubs return `Isa::Scalar`, so a host without the required ISA publishes scalar
rows only.

## Scalar Reference Pseudocode

Phase 2 scalar must preserve `f32::to_bits()` against the pre-kernel scorer.
The scalar block implementation intentionally reuses the same operation order
as the current `sum_query_dequant_bits1_byte_lut_scalar` path:

```text
for lane in 0..32:
    code = codes[lane]
    sum = 0.0f32
    dim = 0
    while dim + 8 <= dimensions:
        row = bits1_byte_lut[code[dim / 8]]
        sum += query_rotated[dim + 0] * row[0]
        ...
        sum += query_rotated[dim + 7] * row[7]
        dim += 8
    while dim < dimensions:
        bit = (code[dim / 8] >> (dim % 8)) & 1
        sum += query_rotated[dim] * bits1_byte_lut[bit][0]
        dim += 1
    out[lane] = finish_scalar_only_estimate(dimensions, 1, sum, code)
return Isa::Scalar
```

This is not optimized, but it locks the contract. Phase 2 tests compare:

- `len < 32` scalar tail vs `PreparedEstimator::estimate_ip_scalar_only`;
- `len == 32` scalar block vs `estimate_ip_bits1_batch`;
- `len > 32` block plus tail vs `estimate_ip_bits1_batch`;
- direct `f32::to_bits()` equality for every candidate.

## SIMD Strategy

The SIMD backends keep two separate stages:

1. A block bit stage that derives exact per-lane sign agreement / disagreement
   counts from packed candidate bytes. This is where popcount instructions live.
2. A finish stage that applies the existing per-candidate scalar metadata and
   score polarity. The finish stage is lane-local and must not be hoisted out
   of candidate order.

The implementation can lower `sum_q_dequant` for bits=1 to a query-weighted
sign-mask accumulation:

```text
dequant(bit) = neg + bit * (pos - neg)
sum_q_dequant = sum_i query[i] * dequant(candidate_bit_i)
              = neg * sum_i query[i]
                + (pos - neg) * sum_i query[i] where candidate_bit_i == 1
```

The popcount-only integer count is useful for Hamming parity and for a
RaBitQ diagnostic assertion, but the production RaBitQ score still needs the
query-weighted positive-bit sum. Phase 2 should add an integer-exact helper for
raw bit counts so Task 95 can share the structure later without changing the
RaBitQ score contract.

### NEON

Backend: `neon::score_block32_neon(...) -> Isa`.

Plan:

- require `target_arch = "aarch64"` and runtime `neon`;
- load candidate bytes for a fixed byte position across groups of lanes;
- use `veorq_u8` where a query sign mask is introduced, then `vcntq_u8` for
  exact bit counts;
- for the weighted RaBitQ sum, either:
  - use the existing byte-LUT pair strategy first for parity, then optimize
    with mask expansion once tests are green; or
  - expand bit masks to `u32` lane masks and accumulate selected query weights
    in `f32x4` groups;
- horizontally reduce per lane, then call the shared scalar finish helper for
  each of the 32 candidates.

Tolerance:

- integer popcount diagnostic counts are exact;
- final score tolerance is <= 4 ULP or `1e-6` relative, with recall byte-equal
  at bench level.

### SVE/SVE2

Backend: `sve::score_block32_sve(...) -> Isa`.

Plan:

- dispatch to SVE/SVE2 only when runtime detection reports the feature;
- remain vector-length agnostic by processing candidate-byte columns under
  predicates instead of assuming `sve2-128` or any other width;
- use SVE/SVE2 byte count (`cnt`) on XOR/sign-mask bytes for exact bit counts;
- accumulate query-weighted sums with predicated loads and reductions;
- return `Isa::Sve2` when the SVE2 branch actually runs, `Isa::Sve` for base
  SVE if a base-SVE implementation lands, and `Isa::Scalar` from fallback stubs.

Evidence packet for the real SVE2 backend must report the measured runtime
vector length verbatim, for example `sve2-128`, and include direct
`[block-kernel-counters]` rows with `isa=sve2`.

### AVX2

Backend: `avx2::score_block32_avx2(...) -> Isa`.

Plan:

- use AVX2 plus FMA only after runtime checks confirm availability;
- avoid AVX-512 VPOPCNTDQ in the AVX2 gate;
- use nibble-LUT `vpshufb` plus `_mm256_sad_epu8` for exact byte popcount, not
  scalar `popcnt` per byte, because this keeps the path within AVX2 and maps
  well to 32-candidate blocks;
- for weighted RaBitQ sums, start from the existing AVX2 bits=1 pair strategy
  in `src/quant/rabitq.rs` if it preserves tolerance with less risk, then
  evolve toward a transposed block layout only if measurement requires it.

Documented choice: nibble-LUT/`pshufb` + `sad_epu8` is the AVX2 popcount
strategy for raw counts; no Harley-Seal first pass unless the nibble LUT fails
the >= 2x scoring-share target.

## AM Registration Plan

The registration phase should add RaBitQ overrides to the relevant
`QuantCodec::score_ip_batch` implementations.

| AM | Current hook | Registration plan |
|---|---|---|
| IVF | `IvfQuantCodec::score_ip_batch` | For `IvfQuantizerProfile::RaBitQ` + `IvfPreparedQuery::RaBitQ`, route `batch.len() >= 32` to `rabitq32`, tails to scalar. |
| DiskANN | `DiskannRaBitQPrefilterCodec` | Add a `score_ip_batch` override using `CandidateBatchScoringSurface::Unknown` until DiskANN gets its own enum value; preserve prefilter score polarity outside the codec. |
| HNSW | `HnswRaBitQScanCodec` | Add a `score_ip_batch` override using `CandidateBatchScoringSurface::Hnsw`; HNSW keeps any traversal polarity conversion outside the codec. |
| SPIRE | `SpireAssignmentScorer::RaBitQ` | Replace the current scalar loop with the same `rabitq32` batch function under `CandidateBatchScoringSurface::Spire`. |

Open design note: `CandidateBatchScoringSurface` currently lacks `Diskann`.
Task 93 Phase 6 should either add `Diskann` to the counter surface or document
why DiskANN remains `Unknown` for this task. The closeout matrix needs direct
DiskANN evidence, so adding the enum value is likely cleaner.

## Counter Contract

For each successful batch:

- whole 32-candidate blocks record `kernel_*` under
  `(surface, QuantCodecKind::RaBitQ, backend_returned_isa)`;
- scalar-only batches and scalar tails record `scalar_*` under
  `(surface, QuantCodecKind::RaBitQ, Isa::Scalar)` through
  `record_block_scalar_score_for`;
- no AM call site derives ISA by calling `current_isa()` after the kernel;
- fallback stubs return `Isa::Scalar`.

Benchmark/measurement packets must cite direct `[block-kernel-counters]` rows,
not only `[task87-counters]` compatibility output.

## Correctness and Measurement Gates

Phase 2:

- scalar `f32::to_bits()` parity against `score_ip_bits1_batch_from_payloads`
  and `estimate_ip_scalar_only`;
- width gates for `<32`, `==32`, and `>32`;
- shape mismatch rejects before counters increment.

Phase 3-5:

- NEON, SVE/SVE2, and AVX2 differential tests against scalar;
- integer-exact raw popcount diagnostics;
- final f32 tolerance <= 4 ULP or `1e-6` relative;
- recall byte-equal at bench level.

Phase 6:

- IVF, DiskANN, HNSW, and SPIRE behavioral parity gates;
- per-AM direct counter rows;
- scalar fallback evidence on hosts without the required ISA.

Phase 7:

- per `(AM x ISA)` results table;
- ADR-076 pointer;
- status flip only after reviewer-approved evidence.

## Safety Boundary

No new `unsafe` outside `src/quant/rabitq32/{neon,sve,avx2}.rs`. Every
intrinsic block needs a `# Safety` doc covering runtime feature detection,
code length, output length, and block width invariants. `mod.rs` and
`scalar.rs` stay safe Rust.
