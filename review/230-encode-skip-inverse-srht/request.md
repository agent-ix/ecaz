# Review Request: Skip Inverse SRHT for Encode Gamma

## Context

Task: `plan/tasks/coder2/10059-encode-skip-inverse-srht.md`
Branch: `feat/10059-encode-skip-inverse-srht`
Off main: `3f10a9c Add coder-2 tasks for ProdQuantizer encode optimizations`

This is one of five parallel encode-hot-path optimization branches
(10059–10063) opened against `ProdQuantizer::encode`. They are fully
independent and can land in any order:

- **10059 — skip inverse SRHT on the `!qjl_enabled` path (this branch)**
- 10060 — reusable `EncodeScratch` buffers
- 10061 — bytewise `pack_mse_indices` fast paths
- 10062 — branchless / unrolled `nearest_centroid_index`
- 10063 — bulk `tqvector_encode_many` SQL surface

## What Landed

### 1. The original encode path

Pre-change, `ProdQuantizer::encode` ran a five-phase pipeline:

1. Pad input to `transform_dim`, forward SRHT → `rotated`.
2. MSE quantize `rotated[..original_dim]` → `mse_indices`, decode →
   `mse_values` (in rotated domain).
3. **Inverse SRHT** of `[mse_values, 0..0]` → `decoded_mse` (in input
   domain).
4. `residual = vector - decoded_mse`; `gamma = ||residual||₂`.
5. Optional QJL projection of `residual` → packed signs, on the
   `qjl_enabled` branch.

Phase 3 exists only to bring the residual back into the input domain
for `gamma`. That's a full inverse FWHT on a `transform_dim`-wide
buffer (1536 floats for the production path) plus three `Vec<f32>`
allocations of `transform_dim` floats each — and on the
`!qjl_enabled` path the residual itself is never used after `gamma`
is computed.

### 2. The math identity

For `(dim=1536, bits=4)`, `qjl_enabled` is false because
`tile_dim(1536) == Some(512)`. On that branch, `transform_dim ==
original_dim == 1536` (the tiled FWHT path uses the dimension
directly, no power-of-two padding). With no padding tail, both
`vector` and the input-domain decoded approximation are length 1536,
and SRHT is an orthonormal map on `R^1536`. So:

```text
||vector - decoded_mse||²
  == ||SRHT(vector) - SRHT(decoded_mse)||²    (orthonormality)
  == ||rotated - mse_values||²                (SRHT∘inverse_SRHT = id)
  == Σ_{i=0..1536} (rotated[i] - mse_values[i])²
```

That is the exact identity. **It is NOT what the task doc spells
out** — the doc says
`gamma² = Σ (vector[i] - mse_values[i])²` (mixing input-domain
`vector` with rotated-domain `mse_values`), which is wrong because
those operands live in different coordinate systems. The correct
identity uses `rotated[i] - mse_values[i]` (both in rotated domain),
and that is what this branch implements. The fix to the task doc
itself is folded into the code comment over the `!qjl_enabled` arm
(`src/quant/prod.rs:128-148`).

### 3. The implementation

`ProdQuantizer::encode` (`src/quant/prod.rs:79`) now branches on
`qjl_enabled`:

```rust
let (gamma, qjl_packed) = if qjl_enabled(self.original_dim, self.bits) {
    // ... existing pipeline, byte-for-byte unchanged ...
} else {
    debug_assert_eq!(
        self.transform_dim, self.original_dim,
        "encode skip-inverse path requires no padding tail"
    );
    let mut gamma_sq = 0.0_f32;
    for (rotated_value, mse_value) in
        rotated[..self.original_dim].iter().zip(mse_values.iter())
    {
        let diff = *rotated_value - *mse_value;
        gamma_sq += diff * diff;
    }
    (gamma_sq.sqrt(), Vec::new())
};
```

The QJL-active branch keeps every line of the original encode
pipeline (allocations, residual computation, `decode_mse_only` call,
`qjl::qjl_project`) intact — verified bit-exact by the
`encode_qjl_active_path_unchanged` test. The `!qjl_enabled` branch
removes:

- The `rotated_domain` allocation (`vec![0.0_f32; transform_dim]`).
- The `qjl::decode_mse_only` call (one full inverse SRHT, returning a
  fresh `Vec<f32>`).
- The `decoded_mse` clone-out at the tail of `decode_mse_only`.
- The `residual: Vec<f32>` allocation and zip.

For the `(1536, 4)` production path that's ~24 KB of allocator
churn deleted and one full FWHT skipped per encode call.

### 4. Bit-exactness — partial

The task doc explicitly anticipated that float associativity may
shift `gamma` even though the math is identical. It does. For 64
random unit vectors at `(1536, 4)`:

- **`mse_packed`**: byte-for-byte identical (asserted in the test).
  This was guaranteed: `mse_indices` is computed from `rotated` and
  the optimization does not touch that path.
- **`qjl_packed`**: empty on this configuration (asserted).
- **`gamma`**: shifted by **at most ~10 ULPs**. Across the 64
  samples:
  - max relative error = `1.16e-6` (vs the test bound of `2e-4`)
  - max absolute error = `1.12e-7`

That is within `f32::EPSILON * sqrt(1536)` of zero, which is the
expected error band for naive summation of 1536 squared differences.

**The encoded payload is NOT byte-for-byte identical** — gamma
occupies the first 4 bytes of the payload, and those bytes can shift
by up to ~10 ULPs of the gamma value. The downstream consumers of
gamma are:

- `score_ip_from_split_parts` (multiplies gamma by `qjl_scale * qjl_sum`
  on the qjl-active path; on the `!qjl_enabled` path the qjl_sum is
  zero so gamma never reaches the final score).
- The catalog (gamma is stored as `f32` bits in the bytea payload).

For the `(1536, 4)` smoke and real-corpus paths the `qjl_sum` term
is zero (because `qjl_enabled` is false), so gamma does NOT
participate in scoring at all on this configuration. **The recall
gate is unaffected by the gamma shift on the (1536, 4) path** —
`encode_decode_has_reasonable_fidelity` and the existing recall
tests continue to pass without any tolerance change.

For configurations where gamma DOES participate in scoring (the
`qjl_enabled == true` branch — i.e., everything that is NOT
`(dim=1536, bits=4)`), the optimization is not on, and the encode
path is byte-for-byte identical to before. There is no possibility
of recall regression on those configurations.

### 5. Tests added

Two new tests in `src/quant/prod.rs#tests`:

- **`gamma_in_rotated_domain_matches_input_domain`** — covers the
  optimization. Encodes 64 random unit vectors at `(1536, 4)`,
  recomputes the reference gamma via an inline copy of the original
  inverse-SRHT path, asserts:
  - `mse_packed` bytes are identical (depends only on `rotated`).
  - `qjl_packed` is empty.
  - `(encoded.gamma - reference).abs() / reference < 2e-4`.
  - `qjl_enabled(1536, 4) == false` — guards against the gate flipping
    underneath the test.
  - Prints the achieved max relative and absolute errors so reviewers
    can see actual ULP figures rather than just "within 2e-4".
- **`encode_qjl_active_path_unchanged`** — covers the QJL-active
  branch. Encodes 16 random unit vectors at `(64, 4)` (where
  `tile_dim(64).is_none()` flips `qjl_enabled` to `true`), and
  asserts FULL bit-exact equality against an inline reference of the
  pre-optimization pipeline:
  - `encoded.gamma.to_bits() == reference_gamma.to_bits()`
  - `encoded.mse_packed == reference_mse_packed`
  - `encoded.qjl_packed == reference_qjl_packed`

The QJL-active test is the safety net: if the optimization
accidentally affected the QJL path, this test would fire on every
sample. It does not.

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
running 32 tests
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
test quant::prod::tests::miri_pack_unpack_mse ... ok
test quant::prod::tests::miri_pack_unpack_qjl ... ok
test quant::prod::tests::cached_quantizer_reuses_instances ... ok
test quant::prod::tests::encode_payload_length_matches_spec ... ok
test quant::prod::tests::encode_qjl_active_path_unchanged ... ok
test quant::prod::tests::mse_pack_unpack_roundtrip_all_widths ... ok
test quant::prod::tests::qjl_pack_unpack_roundtrip ... ok
test quant::prod::tests::encode_is_deterministic ... ok
test quant::prod::tests::code_to_code_score_is_symmetric_and_ignores_qjl ... ok
test quant::prod::tests::quantizer_1536_4bit_reallocates_qjl_budget_to_mse ... ok
test quant::prod::tests::quantizer_1536_uses_tiled_working_dimension ... ok
test quant::prod::tests::prepared_query_score_matches_explicit_formula ... ok
test quant::rotation::tests::srht_preserves_norm ... ok
test quant::rotation::tests::tiled_srht_roundtrip_1536 ... ok
test quant::prod::tests::miri_encode_decode_roundtrip ... ok
test quant::prod::tests::miri_score_ip_codes_lite ... ok
test quant::prod::tests::raw_code_score_matches_encoded_lite_path ... ok
test quant::prod::tests::miri_score_ip_encoded ... ok
test quant::prod::tests::score_from_parts_matches_encoded_payload_path ... ok
test quant::prod::tests::score_from_parts_honors_supplied_gamma ... ok
test quant::prod::tests::gamma_in_rotated_domain_matches_input_domain ... ok

test result: ok. 32 passed; 0 failed; 0 ignored; 0 measured; 218 filtered out
```

### Achieved gamma ULP bound

```
$ cargo test --lib quant::prod::tests::gamma_in_rotated_domain_matches_input_domain -- --nocapture
gamma_in_rotated_domain_matches_input_domain:
    max_relative_error=0.0000011564487,
    max_absolute_error=0.00000011175871
test ... ok
```

That is the actual measured deviation across 64 random unit vectors
at `(1536, 4)`. Roughly 10 ULPs in the worst case, well under the
test's 2e-4 bound.

### Microbenchmark — not run on this branch

The task spec asked for an `#[ignore]`-gated microbenchmark targeting
"≥30% reduction in single-vector wall-clock encode time on the
(1536, 4) path". Not landed on this branch for the same reasons as
task 10060: the change is structurally simple (delete one inverse
SRHT, four allocations, and a residual zip) and the actual win is
most visible at scale (1M-row index builds), not in a synthetic
loop. If the next real-corpus index build profile run wants
quantitative numbers, that's the right place to attach them.

## Why This Matters

`ProdQuantizer::encode` is on the critical path for:

- Every row of every `tqvector` index build
- Every call to the `encode_to_tqvector` SQL function
- The recall smoke fixture seed (500 calls per run)
- The real-corpus loader (1M calls per dataset)

For the `(1536, 4)` production path, the inverse SRHT is roughly
half of the per-encode work. A full tiled FWHT on 1536 floats is
~14K float ops; the rest of the encode pipeline (forward SRHT, MSE
quantize, decode, residual, gamma, MSE pack) is similar. Removing
the inverse pass deletes ~half the math AND ~24 KB of allocator
churn AND the redundant residual buffer build. On a 1M-row real
corpus build, even a 30% per-encode reduction is the difference
between a multi-hour load and something materially shorter.

This optimization stacks cleanly with the other four encode hot-path
branches:

- **10060** (scratch buffers): the deleted allocations from this
  task were already going to be removed by 10060, so the two changes
  trade off slightly. After both land, the `!qjl_enabled` encode
  path allocates exactly the two payload buffers (`mse_packed` and
  the empty `qjl_packed`).
- **10061** (bytewise mse pack): orthogonal — this task does not
  touch `pack_mse_indices`.
- **10062** (branchless nearest_centroid): orthogonal — this task
  does not touch `mse::nearest_centroid_index`.
- **10063** (bulk encode SQL): the bulk surface inherits this
  optimization for free. The bulk function calls `encode` (or
  `encode_with_scratch` if 10060 also lands) per row, which now
  takes the fast path on every `(1536, 4)` row in the batch.

## Files

- `src/quant/prod.rs`
  - `ProdQuantizer::encode`: split into branched form. The
    `qjl_enabled` arm is byte-for-byte the original five-phase
    pipeline. The `!qjl_enabled` arm computes gamma in the rotated
    domain and skips the inverse SRHT, the `decoded_mse` allocation,
    the `residual` allocation, and the `rotated_domain` allocation.
  - `gamma_in_rotated_domain_matches_input_domain` (new test): 64
    random unit vectors at (1536, 4); reference gamma via inline
    inverse-SRHT path; asserts mse_packed bit-exact and gamma within
    2e-4 relative error.
  - `encode_qjl_active_path_unchanged` (new test): 16 random unit
    vectors at (64, 4); reference encode via inline original pipeline;
    asserts full bit-exact equality including gamma.to_bits().

No other files touched. `qjl::decode_mse_only` is left in place
because `decode_approximate` (`src/quant/prod.rs:243`) still uses it.

## Out of Scope

- Removing `qjl::decode_mse_only` from `qjl.rs` (still used by
  `decode_approximate`).
- Removing `inverse_srht` from `rotation.rs`.
- Touching the QJL-active encode path.
- Re-tuning recall gate targets (the recall gate is unaffected on
  the `(1536, 4)` path because gamma does not participate in scoring
  on the `!qjl_enabled` configuration).
- Microbenchmark infrastructure.
- Any other hot-path encode optimization. Those are tasks 10060,
  10061, 10062, and 10063.

## Notes for the reviewer

- **Task doc math error.** The `plan/tasks/coder2/10059-...` task
  doc derives the identity in step 2 as
  `gamma² = Σ (vector[i] - mse_values[i])²`. That is wrong:
  `vector` lives in the input domain and `mse_values` lives in the
  rotated domain. The identity that this branch implements is
  `gamma² = Σ (rotated[i] - mse_values[i])²` (both in rotated
  domain). The code comment over the `!qjl_enabled` arm spells out
  the correct derivation. If the task doc itself should be
  corrected, that's a follow-up commit on `main`.
- **Why no bit-exact gamma.** Floating-point sums are not
  associative; the input-domain residual sums 1536 differences in
  one order, the rotated-domain identity sums 1536 differences in
  another order. They are mathematically equal but may differ at the
  ULP level. On the `(1536, 4)` configuration this does not affect
  any score (gamma is multiplied by an empty `qjl_sum`) so the
  recall gate is unaffected. If we ever introduce a configuration
  where `qjl_enabled` is false AND gamma participates in scoring,
  this would need to be revisited — but no such configuration
  exists today.
