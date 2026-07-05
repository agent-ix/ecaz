# Task: Branchless / Unrolled `nearest_centroid_index`

Motivation: `mse::nearest_centroid_index`
(`src/quant/mse.rs:5`) is the inner loop of MSE quantization. For the
`(dim=1536, bits=4)` real-corpus path it runs 1536 times per encoded
vector, scanning a 16-entry codebook (`mse_bits=4`,
`num_centroids=2^4=16`) with a branching `if distance < best_distance`
update on every step. That's ~24K iterations per encode, all branchy,
all dependent on the previous best. The body is a textbook target for
branchless code or SIMD: the codebook is small enough to keep in
registers, the comparison is `f32` magnitude, and the loop trip count
is a constant the compiler could fully unroll if the codebook size
were exposed.
Priority: batch 3
Status: ready

## Prompt

Replace the branching linear scan in `nearest_centroid_index` with a
branchless update, and provide a fully-unrolled fast path for the
common 16-centroid case (`mse_bits=4`).

### Step 1 — read the current scanner

Read, in order, before touching anything:

- `src/quant/mse.rs:5` (`nearest_centroid_index`) — the function being
  rewritten. Note the current contract: ties go to the lower index
  (because `<` not `<=`). The existing test
  `nearest_centroid_index_prefers_lower_index_on_tie` enforces this.
- `src/quant/mse.rs:18` (`quantize_to_indices`) — the per-dim caller.
- `src/quant/codebook.rs` — confirm the codebook is sorted ascending
  by Lloyd-Max construction. (`grep -n sort src/quant/codebook.rs` and
  read the relevant section.) If it is sorted, an early-exit on
  monotone distance is mathematically valid. **Verify this before
  using it as an optimization.**

### Step 2 — implement a branchless generic scan

Replace the body of `nearest_centroid_index` with a branchless update
that uses a `select` (the LLVM term — in Rust, write it as an
arithmetic blend or use `f32::min`):

```rust
pub fn nearest_centroid_index(codebook: &[f32], value: f32) -> CodeIndex {
    let mut best_index = 0u16;
    let mut best_distance = f32::INFINITY;
    for (index, centroid) in codebook.iter().enumerate() {
        let distance = (value - *centroid).abs();
        let is_better = distance < best_distance;
        // Branchless update: pick whichever element is "better".
        best_distance = if is_better { distance } else { best_distance };
        best_index = if is_better { index as u16 } else { best_index };
    }
    best_index
}
```

Rust's optimizer reliably turns the trailing-`if` form into `cmov` /
`select` when the surrounding code is straightforward. Inspect the
generated assembly with `cargo rustc --release -- --emit asm` against
a small test crate that calls `nearest_centroid_index` in a loop to
confirm. If the compiler is *not* generating branchless code, fall
back to an explicit `f32::to_bits` integer-blend trick — but try the
straightforward form first.

The tie-breaking contract (lower index wins on equal distance) is
preserved by `<` (strict less-than). Do not change to `<=`.

### Step 3 — add a 16-centroid unrolled fast path

The `(1536, 4)` real-corpus path always scans a 16-entry codebook.
Add a specialized entry:

```rust
pub fn nearest_centroid_index_16(codebook: &[f32; 16], value: f32) -> CodeIndex {
    // Fully unrolled 16-way scan with branchless updates.
    // Compiler should reliably vectorize this into a single
    // SIMD min over four f32x4 lanes on x86_64 with avx2.
    let mut best_index = 0u16;
    let mut best_distance = (value - codebook[0]).abs();
    for index in 1..16 {
        let distance = (value - codebook[index]).abs();
        let is_better = distance < best_distance;
        best_distance = if is_better { distance } else { best_distance };
        best_index = if is_better { index as u16 } else { best_index };
    }
    best_index
}
```

`for index in 1..16` with a constant trip count is fully unrollable.
The `&[f32; 16]` array reference (not a slice) lets the compiler
prove the bounds at compile time and lift the bounds checks. Verify
the asm shows no `jmp` for the `is_better` branches (i.e. it became
`cmov`).

Wire `quantize_to_indices` to use the fast path when `codebook.len()
== 16`:

```rust
pub fn quantize_to_indices(codebook: &[f32], rotated: &[f32], dim: usize) -> Vec<CodeIndex> {
    if let Ok(codebook_16) = <&[f32; 16]>::try_from(codebook) {
        rotated[..dim]
            .iter()
            .map(|value| nearest_centroid_index_16(codebook_16, *value))
            .collect()
    } else {
        rotated[..dim]
            .iter()
            .map(|value| nearest_centroid_index(codebook, *value))
            .collect()
    }
}
```

This dispatches to the unrolled path automatically when the cached
quantizer is `(1536, 4)`, with no API change visible to callers.

### Step 4 — explicit SIMD only if branchless is not enough

If the branchless + unrolled path is still slower than expected,
*then* reach for `std::arch::x86_64` intrinsics behind a
`#[cfg(target_feature = "avx2")]` gate. The body is:

- Load 8 `f32` codebook entries into a `__m256`.
- Subtract `value` (broadcast).
- `_mm256_andnot_ps` with the sign mask to take absolute value.
- `_mm256_min_ps` against a running min vector.
- After two halves, horizontal-min the running min vector and find
  the index of the winning lane via comparison.

This is a standard pattern but is fragile across CPU targets and
adds compile-time complexity. **Do not start here.** Land the
branchless version first, measure, and only escalate to intrinsics
if step 5's measurement is below target.

### Step 5 — measure

Add a microbenchmark in `src/quant/mse.rs#tests` (or
`src/quant/prod.rs#tests`) that calls `quantize_to_indices` over
1536 random `f32` inputs against a 16-entry codebook for 100_000
iterations and reports wall clock. Run it against:

- The current branching scan.
- The new branchless generic scan.
- The new unrolled 16-way fast path.

Target: at least **3× speedup** of the unrolled path over the
branching scan on the `(1536, 4)` case. If you only see ~1.5×, the
branch predictor was already hiding the cost — record the number and
decide whether to land for the readability/asm-quality improvement
alone.

### Step 6 — bit-exact regression test

`nearest_centroid_index` returns the same `CodeIndex` for any input
that the branching scan would return, modulo the tie-breaking
contract. Since the branchless update preserves the lower-index-wins
rule, the output is identical for every input.

Add a test that runs both the old branching scan (kept around as
`nearest_centroid_index_branching` in `#[cfg(test)]` only, not in
production) and the new branchless scan over 10_000 random
`(value, codebook)` pairs and asserts they return the same index for
every pair. Same for the 16-way unrolled path against the generic
branchless one. This proves the optimization is bit-exact.

The existing test `nearest_centroid_index_prefers_lower_index_on_tie`
must pass unchanged.

## Design notes

- This task is independent of tasks 10059, 10060, and 10061. They
  stack cleanly: 10059 deletes the inverse SRHT, 10060 reuses
  buffers, 10061 speeds up packing, 10062 speeds up the inner
  quantizer scan. Land in any order.
- The generic branchless path must work for codebook sizes that are
  *not* 16 (e.g. 4, 8, 32) — the existing roundtrip tests at every
  bit width 2..=7 will exercise codebook sizes 4, 8, 16, 32, 64, 128.
- Do not touch `decode_indices`. It is already a single lookup per
  dim and there is nothing to optimize.
- Do not introduce a sorted-codebook early-exit even if the codebook
  is sorted. The early-exit branch is mispredicted on random inputs
  and tends to *hurt* performance compared to a fully-unrolled
  branchless scan. Confirm with measurement if you want to test it,
  but the default is "scan all 16, no early exit".

## Out of scope

- AVX2 / SSE intrinsics (only if step 5 measurement is below target).
- Changing the codebook layout or quantization algorithm.
- Touching the QJL projection inner loop.
- Optimizing `decode_indices` or any other `mse.rs` helper.

## Validate

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --no-default-features --features pg17 quant::mse
cargo test --no-default-features --features pg17 quant::prod
cargo test --no-default-features --features pg17 nearest_centroid
```

All existing `mse.rs` tests must pass unchanged. The new
branchless-vs-branching equivalence test must pass over at least
10_000 random samples. The existing
`mse_pack_unpack_roundtrip_all_widths`, `encode_is_deterministic`,
`encode_payload_length_matches_spec`, and
`quantizer_1536_4bit_reallocates_qjl_budget_to_mse` tests must all
still pass.

Encoded payloads for any `(dim, bits)` config must remain byte-for-
byte unchanged. If they change, the optimization broke the lower-
index-wins tie rule somewhere — diagnose and fix before landing.

Branch from current upstream main. Push branch for review.
