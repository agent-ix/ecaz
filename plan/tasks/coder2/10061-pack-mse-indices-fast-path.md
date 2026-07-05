# Task: Bytewise Fast Path for `pack_mse_indices`

Motivation: `pack_mse_indices` (`src/quant/prod.rs:330`) packs MSE
codebook indices via `write_bits_le`, which loops **one bit at a time**
and redoes the byte/bit math on every iteration. For the `(1536, 4)`
real-corpus path that's 1536 indices × 4 bits = 6144 iterations of
`write_bits_le`, each performing a byte read, mask, OR, and write.
Profiling indicates this loop is a meaningful slice of `encode`'s
per-call cost — far more than its actual information content (768
bytes of output). The bit widths used in production are 2-7, all
small powers and divisors of 8, so a closed-form bytewise pack is
straightforward and ~10× faster than the bit-by-bit loop.
Priority: batch 3
Status: ready

## Prompt

Replace the bit-by-bit `write_bits_le` loop in `pack_mse_indices` with
fast paths for the common bit widths (2, 3, 4, and 5), keeping the
generic loop as a fallback.

### Step 1 — read the current packer

Read, in order, before touching anything:

- `src/quant/prod.rs:330` (`pack_mse_indices`) — the function being
  replaced.
- `src/quant/prod.rs:378` (`write_bits_le`) — the inner per-bit
  helper. Confirm it really does loop one bit per iteration.
- `src/quant/prod.rs:301` (`mse_bits`) — the function that decides
  which bit width applies for a given `(dim, bits)`. The values you
  will hit in production are: `bits=2..=8`, with `mse_bits =
  bits.saturating_sub(1)` if QJL is enabled, or `bits` if disabled.
  Concretely you should expect to see 2, 3, 4, 5, 6, 7 — not 1, not 8.
- `src/quant/prod.rs:330` is mirrored by `unpack_mse_indices` and
  `mse_index_at`. **Do not** rewrite the unpacker; this task is
  pack-only. The packer and unpacker are bit-compatible by construction
  as long as the packer's output bits match `read_bits_le`'s
  little-endian convention.
- The existing test `mse_pack_unpack_roundtrip_all_widths` in
  `src/quant/prod.rs#tests` exercises pack→unpack at every width 1..=7.
  Keep this test passing.

### Step 2 — implement the 4-bit fast path first

The `(1536, 4)` smoke and real-corpus path use 4-bit indices. Two
4-bit indices fit exactly into one byte. The fast path is:

```rust
fn pack_mse_indices_4bit(indices: &[CodeIndex]) -> Vec<u8> {
    let mut packed = vec![0_u8; indices.len().div_ceil(2)];
    for (out_byte, chunk) in packed.iter_mut().zip(indices.chunks(2)) {
        let lo = (chunk[0] & 0x0F) as u8;
        let hi = chunk.get(1).copied().unwrap_or(0) as u8 & 0x0F;
        *out_byte = (hi << 4) | lo;
    }
    packed
}
```

Wire it into `pack_mse_indices` via a `match bits_per_index` dispatch.
Verify the output is byte-exact equal to the current
`write_bits_le`-based output for a 1536-index input — write a one-shot
test that runs both paths and asserts equality.

### Step 3 — add 2, 3, 5 fast paths

Once the 4-bit path is verified:

- **2-bit:** four indices per byte. `(d3 << 6) | (d2 << 4) | (d1 << 2)
  | d0`.
- **3-bit:** eight indices per three bytes (24 bits). Cleanest with a
  small accumulator: walk indices 8 at a time, build a `u32` by
  shifting each `index << (offset * 3)`, write the low 3 bytes.
- **5-bit:** eight indices per five bytes (40 bits). Same pattern: 8
  indices into a `u64`, write low 5 bytes. Verify the byte order
  matches `read_bits_le` little-endian.

For 6 and 7 bits, fall through to the generic per-bit loop. Those
widths are not currently used in production (`bits=8` would imply
`mse_bits=7` on the QJL-active path, and `bits=7` would imply
`mse_bits=6`, but neither is the smoke path) and are not worth a
custom fast path until profiling says they are.

The dispatch should be:

```rust
pub fn pack_mse_indices(indices: &[CodeIndex], bits_per_index: u8) -> Vec<u8> {
    match bits_per_index {
        2 => pack_mse_indices_2bit(indices),
        3 => pack_mse_indices_3bit(indices),
        4 => pack_mse_indices_4bit(indices),
        5 => pack_mse_indices_5bit(indices),
        _ => pack_mse_indices_generic(indices, bits_per_index),
    }
}
```

`pack_mse_indices_generic` is the body of the current
`pack_mse_indices` (the bit-by-bit loop) renamed.

### Step 4 — exhaustive bit-exact regression test

Add a regression test in `src/quant/prod.rs#tests` that, for each
`bits_per_index` in `2..=7`, generates `1536` random indices in
`0..(1u16 << bits_per_index)`, packs via the dispatched path, packs
via the generic path, and asserts the two outputs are byte-equal.
This guarantees the fast paths produce literally the same packed bytes
as the bit-by-bit loop, which is what makes it safe to land without
re-baselining recall.

Also keep `mse_pack_unpack_roundtrip_all_widths` running unchanged —
it tests the pack→unpack identity, which is the user-visible
correctness contract.

### Step 5 — measure

Add an `#[ignore]`-gated microbenchmark in `src/quant/prod.rs#tests`
that calls `pack_mse_indices` with 1536 4-bit indices in a 100_000-
iteration `Instant` loop on both paths and prints the wall clock for
each. Target: at least **5× speedup** for the 4-bit path. If the
speedup is below 3×, the bit loop was not as bad as profiling
suggested — record the actual number and decide whether the
complexity is worth it.

## Design notes

- The fast paths must produce **bit-exact** output equal to the
  generic path. This is the entire reason for the regression test in
  step 4. Any divergence — even one byte — will silently change
  encoded payloads, change recall summaries, and break the smoke
  test's "byte-identical reruns" assertion.
- The unpacker (`mse_index_at`) is intentionally untouched. It is
  called once per dim during scoring, not per-batch, and the
  per-call overhead is small relative to the LUT lookup it gates. A
  future task could optimize it; this task does not.
- Do not change the function signature of `pack_mse_indices`. Callers
  that pass `bits_per_index` outside `2..=7` (e.g. tests at width 1
  or 6) must continue to work via the generic fallback.
- Do not introduce SIMD. The byte-shift fast paths are already memory-
  bound for 768-byte outputs; SIMD adds complexity without measurable
  gain at this output size.

## Out of scope

- Optimizing `unpack_mse_indices` or `mse_index_at`.
- Optimizing `pack_qjl_signs` (already byte-aligned and trivially
  fast).
- Changing the codebook layout, the bit-width selection rule, or any
  of the math.
- 6-bit and 7-bit fast paths.

## Validate

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --no-default-features --features pg17 quant::prod
cargo test --no-default-features --features pg17 mse_pack_unpack_roundtrip_all_widths
```

The new exhaustive bit-exact regression test must pass for every
width in `2..=7`. The existing roundtrip test must still pass. Encode
output for any `(dim, bits)` config must be byte-for-byte unchanged.

Branch from current upstream main. Push branch for review.
