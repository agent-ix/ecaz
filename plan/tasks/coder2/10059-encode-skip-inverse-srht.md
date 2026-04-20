# Task: Compute Encode `gamma` Without an Inverse SRHT

Motivation: `ProdQuantizer::encode` (`src/quant/prod.rs:79`) currently
runs a forward SRHT, MSE-quantizes the rotated coordinates, then runs an
**inverse** SRHT (`qjl::decode_mse_only`, `src/quant/qjl.rs:10`) just so
the residual `vector - decoded_mse` can be taken in the original domain
to compute `gamma = ||residual||`. The inverse pass exists only to put
the residual back into the input domain; the math does not require it.
SRHT is orthonormal, so `||residual_original|| == ||residual_rotated||`,
and the residual in the rotated domain is `rotated[i] - mse_values[i]`
for `i in 0..original_dim` and `rotated[i]` for `i in original_dim..
transform_dim` (where the padding tail mixes back in via SRHT). Removing
the inverse pass deletes one full FWHT, three `Vec<f32>` allocations of
`transform_dim` floats, two copies, and a residual zip — roughly half
the per-encode work for the `(1536, 4)` smoke / real-corpus path. At ~16
ms per encode × 1M rows on a real corpus build, that's the difference
between a multi-hour load and something materially shorter. This
optimization is conditional on `!qjl_enabled` (the path that does not
need the residual in the original domain for QJL projection).
Priority: batch 3
Status: ready

## Prompt

Replace the encode pipeline's forward-then-inverse SRHT with a single
forward SRHT plus a `gamma` computation in the rotated domain, on the
`!qjl_enabled` path only.

### Step 1 — read the current pipeline

Read, in order, before touching anything:

- `src/quant/prod.rs:79` (`ProdQuantizer::encode`) — the function being
  changed.
- `src/quant/qjl.rs:10` (`decode_mse_only`) — the inverse SRHT call this
  task removes. Confirm it is *only* used by `encode` and not by any
  other code path. (`grep -n decode_mse_only src/`.)
- `src/quant/rotation.rs:35` (`srht`) and `src/quant/rotation.rs:49`
  (`inverse_srht`) — the orthonormal pair. Confirm they are exact
  inverses on power-of-two-len inputs and on the tiled-1536 path.
- `src/quant/prod.rs:297` (`qjl_enabled`) — the gate that decides
  whether the QJL projection runs. The optimization is conditional on
  this being `false`. For `(dim=1536, bits=4)` it is `false`; for the
  general `(dim, bits)` case where `bits != 4 || tile_dim(dim).is_none()`
  it is `true` and the residual in the original domain is still needed
  by `qjl::qjl_project(&residual, &self.qjl_signs)`.

### Step 2 — derive and assert the math identity

`gamma` is `||vector - decoded_mse||₂`. SRHT is an orthonormal map on
the `transform_dim`-wide padded space, so for any pair `a, b` in that
space, `||a - b||₂ == ||SRHT(a) - SRHT(b)||₂`. Apply this with
`a = padded(vector)` and `b = padded_decoded_mse_in_original_domain`
(zero-padded to `transform_dim`):

- `SRHT(a) = rotated` (already computed)
- `SRHT(b)` = the rotated representation of `[mse_values, 0, ..., 0]`,
  which is exactly what `decoded_mse_only`'s inverse pass produces and
  then re-rotates — but we never need to materialize it. Instead,
  observe that the "rotated coordinates of `b`" are obtained by
  applying SRHT to `[mse_values, 0..0]` directly. We know
  `SRHT([mse_values, 0..0])` is **not** simply `[mse_values, 0..0]`,
  so the naive "subtract in rotated space" identity does not hold
  unless we also account for the zero-tail mixing.

Therefore the cleanest identity is **NOT** "subtract in rotated space".
It is: compute the residual in the **input domain** without doing a
full inverse SRHT, by forming `padded_decoded = [mse_values, 0..0]`,
**rotating it forward** with the same signs to get `rotated_decoded`,
and then taking `gamma = ||rotated - rotated_decoded||₂`. That replaces
one inverse SRHT with one forward SRHT — same cost, no win.

The actual win is different: **fold the SRHT linearity**. SRHT is
linear, so `rotated - SRHT([mse_values, 0..0]) == SRHT(padded(vector) -
[mse_values, 0..0])`. Applying SRHT only changes coordinates, not the
norm, so:

```text
gamma² = || padded(vector) - [mse_values, 0..0] ||²
       = Σ_{i in 0..original_dim} (vector[i] - mse_values[i])²
         + Σ_{i in original_dim..transform_dim} (0 - 0)²
       = Σ_{i in 0..original_dim} (vector[i] - mse_values[i])²
```

That is: **the residual norm in the input domain is just the elementwise
difference between the original input vector and the decoded MSE values
over the first `original_dim` coordinates, with zero contribution from
the padding tail**. No SRHT, forward or inverse, is needed at all to
compute it.

Crucially this means the optimization does NOT need `decode_mse_only`
or `inverse_srht` on this path. It computes `gamma` straight from
`vector` (the function's input) and `mse_values` (already computed
upstream).

### Step 3 — implement on the `!qjl_enabled` path

In `ProdQuantizer::encode`, branch on `qjl_enabled(self.original_dim,
self.bits)`:

- **`true` (QJL active):** keep the existing pipeline byte-for-byte
  unchanged. The QJL projection reads `residual` in the original domain
  and there is no shortcut.
- **`false` (QJL disabled, the `1536/4` smoke + real-corpus path):**
  skip `rotated_domain` allocation, skip `qjl::decode_mse_only`, skip
  the `residual: Vec<f32>` allocation. Compute `gamma` directly:

  ```rust
  let mut gamma_sq = 0.0_f32;
  for (input, approx) in vector.iter().zip(mse_values.iter()) {
      let diff = *input - *approx;
      gamma_sq += diff * diff;
  }
  let gamma = gamma_sq.sqrt();
  ```

  Set `qjl_packed = Vec::new()` (matching the existing branch), and
  build `EncodedTq` exactly as today.

Do not delete `decode_mse_only` from `qjl.rs` even if it becomes
unused on the encode path — leave it; it is small, documented, and
removable in a future cleanup once nothing references it. If `cargo
build` then warns about dead code, gate it under `#[allow(dead_code)]`
locally rather than threading a deletion through this task.

### Step 4 — bit-exact regression test

Float addition is not associative. The current path computes `gamma` as
`Σ (vector[i] - decoded_mse_inverse_srht[i])²` where `decoded_mse_inverse_srht`
is the result of an inverse SRHT (which sums many terms). The new path
computes `gamma` as `Σ (vector[i] - mse_values[i])²` directly. **These
are mathematically equal but may differ by 1 ULP in float.**

Add a regression test alongside the existing
`quantizer_1536_4bit_reallocates_qjl_budget_to_mse` test in
`src/quant/prod.rs` that:

1. Generates ~64 random unit vectors at `(dim=1536, bits=4, seed=42)`.
2. Encodes each on the new path.
3. Compares the produced `EncodedTq` against a vector of golden encoded
   payloads captured from the old path **before this change lands**.
4. Asserts byte-exact equality of `mse_packed` and `qjl_packed` for
   every sample.
5. Asserts `gamma` differs by at most 1 ULP (use
   `(new.gamma - old.gamma).abs() < f32::EPSILON * old.gamma.max(1.0)`
   or compare bit patterns directly with `f32::to_bits`).

If the gamma value bit-pattern is unstable, the optimization is still
defensible — but the recall smoke
(`test_ec_hnsw_graph_scan_recall_external_smoke_500`) and any A4 recall
gate runs **must be re-baselined** on the new gamma values. Document
this in the review packet with before/after recall numbers, and update
any pinned recall summary digests.

The golden payloads are easiest captured by running the existing
encode path once, hex-dumping the bytes per sample, and pasting them
into the test as `const`s. Do not generate them at runtime — that
defeats the whole purpose of the regression test.

### Step 5 — measure

Capture before/after wall-clock for `ProdQuantizer::encode` on a
single 1536-dim vector via a microbenchmark. The repo's
`scripts/bench_sql_latency.sh` is too coarse for this; add a small
inline `#[bench]`-style harness in `src/quant/prod.rs#tests` that uses
`std::time::Instant` over 10_000 encodes and prints the average.
Remove the harness before commit (or gate it under `#[ignore]`).

Target: at least **30% reduction** in single-vector encode time on the
`(1536, 4)` path. If you do not see at least that, the inverse SRHT was
not the dominant cost on your machine — record what you measured, and
hand the result to coder-1 to decide whether to land anyway as a
correctness simplification.

## Design notes

- This is an `encode`-only optimization. Do not touch
  `prepare_ip_query`, `score_ip_*`, `decode_approximate`, or any of the
  `pack_*` / `unpack_*` helpers.
- The QJL-active path stays byte-for-byte identical. There is no win
  there because `qjl_project` reads the residual in the original domain
  and the inverse SRHT is genuinely needed.
- Do not change `qjl_enabled`'s definition. The branching predicate is
  correct as-is.
- Do not introduce SIMD on this task. That's task 10062.
- Do not introduce scratch buffers on this task. That's task 10060.
  This task should be additive to both: when 10060 lands, the deleted
  allocations from this task are already gone, and 10060's scratch
  buffers shrink slightly.

## Out of scope

- Removing `qjl::decode_mse_only` from `qjl.rs`.
- Removing `inverse_srht` from `rotation.rs`.
- Touching the QJL-active encode path.
- Re-tuning recall gate targets if `gamma` shifts. That is a follow-up
  review with its own packet.
- Bulk encode API. That's task 10063.

## Validate

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --no-default-features --features pg17 quant::prod
cargo test --no-default-features --features pg17 quantizer_1536_4bit
```

The new bit-exact regression test must pass. The existing
`encode_is_deterministic`, `encode_decode_has_reasonable_fidelity`,
`encode_payload_length_matches_spec`, and
`quantizer_1536_4bit_reallocates_qjl_budget_to_mse` tests must all
still pass without modification.

If this task changes any byte of any encoded payload at `(1536, 4)`,
attach the diff and the recall delta to the review packet. If it does
not, attach a "byte-identical, gamma stable to N ULPs over 64 samples"
note with the actual N.

Branch from current upstream main. Push branch for review.
