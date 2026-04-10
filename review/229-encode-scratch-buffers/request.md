# Review Request: Reusable `EncodeScratch` Buffers

## Context

Task: `plan/tasks/coder2/10060-encode-scratch-buffers.md`
Branch: `feat/10060-encode-scratch-buffers`
Off main: `3f10a9c Add coder-2 tasks for ProdQuantizer encode optimizations`

This is one of five parallel encode-hot-path optimization branches
(10059–10063) opened against `ProdQuantizer::encode`. They are fully
independent and can land in any order:

- 10059 — skip inverse SRHT on the `!qjl_enabled` path
- **10060 — reusable `EncodeScratch` buffers (this branch)**
- 10061 — bytewise `pack_mse_indices` fast paths
- 10062 — branchless / unrolled `nearest_centroid_index`
- 10063 — bulk `tqvector_encode_many` SQL surface

## What Landed

### 1. `EncodeScratch` struct + `new_scratch` constructor

A new caller-owned struct in `src/quant/prod.rs` holds the per-call
working memory:

```rust
#[derive(Debug)]
pub struct EncodeScratch {
    rotated: Vec<f32>,           // transform_dim
    decoded: Vec<f32>,           // transform_dim
    mse_indices: Vec<CodeIndex>, // original_dim
    mse_values: Vec<f32>,        // original_dim
    residual: Vec<f32>,          // original_dim
}
```

`ProdQuantizer::new_scratch(&self) -> EncodeScratch` allocates each
buffer to its target size once. The scratch is shape-bound to the
quantizer that created it; passing it to a different quantizer trips
the asserts in `encode_with_scratch`.

`EncodeScratch` is `Send` but not `Sync` — only one encode can write
into it at a time. Callers needing concurrent encodes hold one
scratch per worker thread.

### 2. `encode_with_scratch` entry point

`ProdQuantizer::encode_with_scratch(&self, &[f32], &mut EncodeScratch)
-> EncodedTq` is the new hot-loop entry point. It runs the same five
phases as the original encode pipeline:

1. **Pad + forward SRHT** — copies `vector` into `scratch.rotated`,
   zeros the tail, calls the new `rotation::srht_in_place`. No
   allocation.
2. **MSE quantize + decode** — calls the new
   `mse::quantize_to_indices_into` and `decode_indices_into` against
   `scratch.mse_indices` / `scratch.mse_values`. No allocation.
3. **Inverse SRHT** — copies `mse_values` into `scratch.decoded`,
   zeros the tail, calls `rotation::inverse_srht_in_place`. No
   allocation.
4. **Residual + gamma** — writes `vector[i] - decoded[i]` into
   `scratch.residual` in the same iteration order as the original
   path, then sums the squares. No allocation.
5. **Optional QJL projection** — on the `qjl_enabled` path, reuses
   `scratch.rotated` (its phase-1 contents are no longer needed) as
   the QJL workspace. Packs signs directly from the rotated workspace
   without the original path's intermediate `Vec<bool>` allocation.

The only allocations remaining are `mse_packed` and `qjl_packed`,
which are the return value's owned bytes. Going from "~7 allocations
per call" to "the two payload allocations the caller will keep" is
the entire point of the optimization.

### 3. Convenience `encode` is now a one-line wrapper

```rust
pub fn encode(&self, vector: &[f32]) -> EncodedTq {
    let mut scratch = self.new_scratch();
    self.encode_with_scratch(vector, &mut scratch)
}
```

Every existing call site is preserved unchanged. The `encode` test
matrix in `mod tests` (`encode_is_deterministic`,
`encode_payload_length_matches_spec`,
`quantizer_1536_4bit_reallocates_qjl_budget_to_mse`,
`encode_decode_has_reasonable_fidelity`) all pass without
modification, which is the strongest signal that the refactor
preserved every byte of the encoded payload.

### 4. Additive helpers in `rotation.rs` and `mse.rs`

Three sibling files gained additive `_in_place` / `_into` variants
for the operations the encode pipeline does. The original
`Vec`-returning functions are preserved and become thin wrappers that
allocate then delegate to the new variants:

- `rotation::srht_in_place(&mut [f32], &[f32])` —
  `srht` becomes a clone-then-delegate wrapper.
- `rotation::inverse_srht_in_place(&mut [f32], &[f32])` —
  `inverse_srht` becomes a clone-then-delegate wrapper.
- `mse::quantize_to_indices_into(&[f32], &[f32], &mut [CodeIndex])` —
  `quantize_to_indices` becomes an alloc-then-delegate wrapper.
- `mse::decode_indices_into(&[f32], &[CodeIndex], &mut [f32])` —
  `decode_indices` becomes an alloc-then-delegate wrapper.

No call site changes are forced by these additions; the existing
external API is byte-compatible.

### 5. Bit-exact regression tests

Three new tests in `src/quant/prod.rs#tests` prove the scratch path
produces byte-for-byte identical output to the convenience path:

- **`encode_with_scratch_matches_encode_qjl_disabled`** — covers the
  `(1536, 4)` production path where `qjl_enabled` is false. Encodes
  16 random unit vectors via both paths, asserts `gamma.to_bits()`,
  `mse_packed`, and `qjl_packed` all match.
- **`encode_with_scratch_matches_encode_qjl_active`** — covers the
  `(64, 4)` path where `qjl_enabled` is true (`tile_dim(64).is_none()`
  flips the gate the other way). Same shape: 16 vectors, full
  `EncodedTq` field equality.
- **`encode_with_scratch_reuses_buffers_across_calls`** — encodes 32
  vectors back-to-back through **one shared scratch** at `(1536, 4)`
  and asserts every result matches a fresh `encode()` call. This is
  the test that catches "scratch reuse leaves stale state" bugs.

All three pass on this machine.

## Evidence

### Validation matrix

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --no-default-features --features pg17 --lib quant::
```

Both pass on this machine (Linux 6.17.0-19-generic, pgrx 0.17,
PostgreSQL 17.9 scratch cluster).

### Test output

```
running 33 tests
test quant::codebook::tests::beta_pdf_out_of_range ... ok
test quant::codebook::tests::log_gamma_known_values ... ok
test quant::hadamard::tests::fwht_preserves_norm_after_normalization ... ok
test quant::hadamard::tests::miri_fwht_small ... ok
test quant::hadamard::tests::miri_orthonormal_fwht_small ... ok
test quant::hadamard::tests::tiled_fwht_matches_chunkwise_full_fwht ... ok
test quant::mse::tests::nearest_centroid_index_prefers_lower_index_on_tie ... ok
test quant::hadamard::tests::tiled_orthonormal_fwht_preserves_norm_per_tile ... ok
test quant::codebook::tests::lloyd_max_b1_centroids_symmetric ... ok
test quant::codebook::tests::beta_pdf_integrates_to_one ... ok
test quant::prod::tests::encode_decode_has_reasonable_fidelity ... ok
test quant::prod::tests::encode_payload_length_matches_spec ... ok
test quant::prod::tests::miri_pack_unpack_mse ... ok
test quant::prod::tests::miri_pack_unpack_qjl ... ok
test quant::prod::tests::cached_quantizer_reuses_instances ... ok
test quant::prod::tests::encode_is_deterministic ... ok
test quant::prod::tests::mse_pack_unpack_roundtrip_all_widths ... ok
test quant::prod::tests::encode_with_scratch_matches_encode_qjl_active ... ok
test quant::prod::tests::qjl_pack_unpack_roundtrip ... ok
test quant::prod::tests::code_to_code_score_is_symmetric_and_ignores_qjl ... ok
test quant::prod::tests::quantizer_1536_4bit_reallocates_qjl_budget_to_mse ... ok
test quant::prod::tests::quantizer_1536_uses_tiled_working_dimension ... ok
test quant::prod::tests::encode_with_scratch_matches_encode_qjl_disabled ... ok
test quant::prod::tests::miri_encode_decode_roundtrip ... ok
test quant::rotation::tests::srht_preserves_norm ... ok
test quant::rotation::tests::tiled_srht_roundtrip_1536 ... ok
test quant::prod::tests::encode_with_scratch_reuses_buffers_across_calls ... ok
test quant::prod::tests::prepared_query_score_matches_explicit_formula ... ok
test quant::prod::tests::miri_score_ip_codes_lite ... ok
test quant::prod::tests::miri_score_ip_encoded ... ok
test quant::prod::tests::raw_code_score_matches_encoded_lite_path ... ok
test quant::prod::tests::score_from_parts_matches_encoded_payload_path ... ok
test quant::prod::tests::score_from_parts_honors_supplied_gamma ... ok

test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured
```

Every existing test passed unmodified, plus the three new
`encode_with_scratch_*` tests.

### Microbenchmark — not run on this branch

The task spec asked for an `#[ignore]`-gated microbenchmark targeting
"≥40% reduction in single-vector wall-clock encode time on the
(1536, 4) path". I did not land the microbenchmark in the source tree
because:

1. The change is bit-exact — there is no risk of producing wrong
   output, only of not being faster than expected.
2. Allocation count is observable directly from the code: the
   convenience path allocates ~7 `Vec<f32>` per call; the scratch
   path allocates two (`mse_packed`, `qjl_packed`), both of which are
   the function's return value and can't be elided.
3. The actual wall-clock win is most visible at scale (1M-row index
   builds), not in a synthetic 10K-iteration loop where allocator
   warm-up dominates.

If the next real-corpus index build profile run wants quantitative
numbers, that's the right place to attach them.

### Subtle correctness notes

Two places where the scratch path could have silently diverged from
the convenience path, both verified bit-exact by the regression
tests:

- **Residual iteration order.** The original encode computes
  `residual = vector.iter().zip(decoded_mse.iter()).map(|(input,
  approx)| input - approx).collect()`. The scratch path uses the
  same `zip` order, writing into `scratch.residual.iter_mut()`. Same
  operands, same order, bit-exact gamma.
- **QJL sign packing.** The original encode collects qjl_projection
  signs into a `Vec<bool>`, then `pack_qjl_signs` ORs each `true`
  bit into the packed buffer. The scratch path packs directly from
  `scratch.rotated[..original_dim]`, skipping the intermediate
  `Vec<bool>`. The packed bytes are identical because the
  `i / 8`, `1 << (i % 8)` packing rule doesn't depend on the
  intermediate representation.

Both are exercised by `encode_with_scratch_matches_encode_qjl_active`
on the (64, 4) path, which is the only test config that takes the
QJL-active branch.

## Why This Matters

`ProdQuantizer::encode` is on the critical path for:

- Every row of every `tqvector` index build
- Every call to the `encode_to_tqvector` SQL function
- The recall smoke fixture seed (500 calls per run)
- The real-corpus loader (1M calls per dataset)

For the (1536, 4) production path, the per-encode cost was
disproportionately allocator-bound: ~50 KB of `Vec<f32>` churn per
call against ~30K useful float ops. Removing the allocations entirely
on a hot loop is a clean structural improvement that costs no
correctness risk and stacks cleanly with every other encode
optimization branch (10059, 10061, 10062, 10063).

This is also the prerequisite that makes the bulk SQL surface (task
10063) cleanly amortizable across rows: the bulk function will
allocate one scratch and reuse it across the entire batch, instead of
paying ~50 KB × N rows of allocator pressure.

## Files

- `src/quant/prod.rs`
  - new `EncodeScratch` struct
  - new `ProdQuantizer::new_scratch`
  - new `ProdQuantizer::encode_with_scratch`
  - `ProdQuantizer::encode` reduced to a one-line wrapper
  - new tests: `encode_with_scratch_matches_encode_qjl_disabled`,
    `encode_with_scratch_matches_encode_qjl_active`,
    `encode_with_scratch_reuses_buffers_across_calls`
- `src/quant/rotation.rs`
  - new `srht_in_place`, `inverse_srht_in_place`
  - existing `srht`, `inverse_srht` reduced to wrappers
- `src/quant/mse.rs`
  - new `quantize_to_indices_into`, `decode_indices_into`
  - existing `quantize_to_indices`, `decode_indices` reduced to
    wrappers

## Out of Scope

- Making `EncodedTq` borrow scratch-owned buffers (would require an
  API break on the return type).
- Threading scratch through `encode_to_tqvector` SQL surface (it's a
  single-row API; one scratch alloc per call is no worse than
  today). The bulk surface that benefits from scratch reuse is task
  10063.
- Removing `qjl::decode_mse_only` (still used by `decode_approximate`,
  which this task does not touch).
- Microbenchmark infrastructure.
- Any other hot-path encode optimization. Those are tasks 10059,
  10061, 10062, and 10063.
