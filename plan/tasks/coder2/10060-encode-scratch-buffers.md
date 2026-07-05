# Task: Reusable Scratch Buffers for `ProdQuantizer::encode`

Motivation: `ProdQuantizer::encode` (`src/quant/prod.rs:79`) allocates
~6 fresh `Vec<f32>` of size `transform_dim` plus a `Vec<CodeIndex>` of
size `original_dim` per call. For the `(1536, 4)` real-corpus build
path that's roughly 50 KB of allocator churn per encoded vector, called
~1M times during a real-corpus index build. The allocator and the
copies it forces are likely a substantial fraction of the per-encode
cost (current measurement: ~16 ms per call, only ~3 Mflop/s of useful
math, which strongly suggests memory traffic and allocation are
dominating). Threading scratch buffers through `encode` removes those
allocations entirely on a hot loop without changing any encoded
output.
Priority: batch 3
Status: ready

## Prompt

Add a reusable `EncodeScratch` struct that owns the per-call working
buffers, and expose a `ProdQuantizer::encode_with_scratch` entry point
that callers in hot loops can use to amortize allocation across many
vectors. Keep the existing `encode(&self, &[f32]) -> EncodedTq` API
working unchanged.

### Step 1 — read the current pipeline

Read, in order, before touching anything:

- `src/quant/prod.rs:79` (`ProdQuantizer::encode`) — list every `Vec`
  it allocates per call. The current count is at least: `padded`,
  `rotated` (cloned inside `srht`), `mse_indices`, `mse_values`,
  `rotated_domain`, `decoded_mse` (returned by `decode_mse_only` after
  another inverse SRHT clone), and `residual`. Plus `mse_packed` and
  `qjl_packed` inside `EncodedTq` itself.
- `src/quant/rotation.rs:35` (`srht`) and
  `src/quant/rotation.rs:67` (`pad_input`) — both currently take
  `&[f32]` and return a fresh `Vec<f32>`. The optimization needs
  in-place variants (or `&mut [f32]` variants).
- `src/quant/mse.rs:18` (`quantize_to_indices`) and
  `src/quant/mse.rs:25` (`decode_indices`) — both return fresh `Vec`s.
  These can either grow `&mut Vec<...>` parameters or write into a
  pre-sized `&mut [...]` slice.
- `src/quant/qjl.rs:10` (`decode_mse_only`) — only relevant on the
  QJL-active path; on the `1536/4` path it goes away after task 10059.
  This task should still handle both branches correctly.

### Step 2 — design the scratch struct

Add a new `EncodeScratch` struct in `src/quant/prod.rs` that owns the
per-call working memory:

```rust
pub struct EncodeScratch {
    /// Padded + sign-flipped + FWHT-transformed working buffer.
    /// Length == transform_dim. Reused across encodes; the pad-tail
    /// region is rezeroed at the start of each call.
    rotated: Vec<f32>,
    /// MSE quantizer code indices. Length == original_dim.
    mse_indices: Vec<CodeIndex>,
    /// Decoded MSE values in original-domain ordering.
    /// Length == original_dim.
    mse_values: Vec<f32>,
    /// Working buffer for the QJL-active inverse-SRHT round trip.
    /// Length == transform_dim. Empty (capacity 0) when QJL is
    /// disabled for the cached quantizer's (dim, bits).
    qjl_workspace: Vec<f32>,
    /// Residual buffer (only used on the QJL-active path).
    /// Length == original_dim. Empty when QJL is disabled.
    residual: Vec<f32>,
}
```

Provide a `ProdQuantizer::new_scratch(&self) -> EncodeScratch`
constructor that allocates each buffer to its target size *once*. The
quantizer knows its own `transform_dim`, `original_dim`, and
`qjl_enabled` state, so it picks the correct sizes and skips
QJL-only buffers when QJL is off.

The scratch struct is `Send` but **not** `Sync` — only one encode can
write into it at a time. Document that callers needing concurrent
encodes should hold one `EncodeScratch` per worker thread.

### Step 3 — add the `encode_with_scratch` entry point

Add:

```rust
pub fn encode_with_scratch(
    &self,
    vector: &[f32],
    scratch: &mut EncodeScratch,
) -> EncodedTq {
    // ... same pipeline as encode(), but writes into scratch.* instead
    // of allocating fresh Vec<f32>s.
}
```

The function:

- Asserts `vector.len() == self.original_dim` (same as today).
- Asserts the scratch buffers have the expected lengths (catches the
  case where a scratch was created against a different quantizer).
- Reuses `scratch.rotated`: the first `original_dim` elements get
  filled by copying `vector`; the tail region (if any) gets zeroed.
  Then sign-flip in place, then FWHT in place.
- Reuses `scratch.mse_indices` and `scratch.mse_values` instead of
  allocating fresh `Vec`s in `mse::quantize_to_indices` /
  `mse::decode_indices`. This requires either new `_into` variants of
  those functions (`quantize_into(&[f32], &[f32], &mut [CodeIndex])`,
  `decode_into(&[f32], &[CodeIndex], &mut [f32])`) or inlining the
  body of each one at the call site. Prefer the `_into` variants —
  they keep the math testable in isolation.
- On the QJL-active path, reuses `scratch.qjl_workspace` for the
  inverse-SRHT round trip and `scratch.residual` for the residual.
- Returns `EncodedTq { gamma, mse_packed, qjl_packed }`. Note that
  `EncodedTq.mse_packed` and `qjl_packed` themselves still allocate —
  this task does not try to make those reusable, because they cross
  the function boundary as the return value. (A future task could
  make `EncodedTq` borrow scratch-owned buffers, but that's an API
  break and out of scope here.)

### Step 4 — keep the old `encode` working

`fn encode(&self, vector: &[f32]) -> EncodedTq` becomes a one-liner
that allocates a fresh scratch and forwards:

```rust
pub fn encode(&self, vector: &[f32]) -> EncodedTq {
    let mut scratch = self.new_scratch();
    self.encode_with_scratch(vector, &mut scratch)
}
```

This preserves every existing call site and every existing test
without modification. Encoded output must remain byte-for-byte
identical (the scratch path is just a different memory layout for the
same arithmetic).

### Step 5 — wire `encode_to_tqvector` if it stays single-vector

`encode_to_tqvector` (`src/lib.rs:430`) is the SQL surface called once
per row. As long as it stays a single-vector function, it should
continue to use the convenience `encode` wrapper — the per-call
scratch allocation it pays is no worse than today. **Do not** try to
hoist a thread-local scratch into the `#[pg_extern]` function; pgrx
backends are short-lived and the lifetime contract is not worth the
risk for a single-row API. The bulk SQL surface that benefits from
scratch reuse is task 10063's job.

### Step 6 — bit-exact regression test

The mathematical operations are unchanged; only the buffer ownership
is different. Encoded output must therefore be bit-exact. Add a test
in `src/quant/prod.rs#tests`:

```rust
#[test]
fn encode_with_scratch_matches_encode() {
    let quantizer = ProdQuantizer::new(1536, 4, 42);
    let mut scratch = quantizer.new_scratch();
    for sample_seed in 0..16 {
        let vector = random_unit_vector(1536, sample_seed);
        let plain = quantizer.encode(&vector);
        let scratched = quantizer.encode_with_scratch(&vector, &mut scratch);
        assert_eq!(plain.gamma.to_bits(), scratched.gamma.to_bits());
        assert_eq!(plain.mse_packed, scratched.mse_packed);
        assert_eq!(plain.qjl_packed, scratched.qjl_packed);
    }
}
```

Cover both the QJL-disabled path (`(1536, 4)`) and a QJL-enabled path
(`(64, 4)` or similar — anything where `qjl_enabled` is `true`) so
both branches of the new function are exercised.

### Step 7 — measure

Microbenchmark `encode_with_scratch` against `encode` over 10_000
iterations on the `(1536, 4)` path, sharing one scratch across all
iterations of the new path and forcing a fresh allocation per iteration
on the old path. Use `std::time::Instant` in an `#[ignore]`-gated test
in `src/quant/prod.rs#tests`. Report wall-clock and allocations per
iteration (the allocations metric can come from a `#[global_allocator]`
counting wrapper, or just from "the new path does zero allocs in steady
state, the old path does 7 — confirm by reading the code"). Remove the
microbenchmark before commit unless you gate it under `#[ignore]` with
a clear name.

Target: at least **40% reduction** in single-vector wall-clock encode
time on the `(1536, 4)` path against the unchanged `encode()` baseline.
If you also stack this on top of task 10059 (skip inverse SRHT) the
combined target is "as fast as a hand-rolled inline encode".

## Design notes

- The scratch struct must be quantizer-shape-specific. Calling
  `encode_with_scratch` with a scratch built for a different `(dim,
  bits, seed)` is a programming error and should panic on the assert
  in step 3, not silently produce wrong output.
- Do not introduce a thread-local or `OnceCell`-cached scratch. That
  hides the allocation contract from the caller and creates a hidden
  race condition with concurrent encodes. Make the caller pass `&mut
  EncodeScratch` explicitly.
- This task is independent of task 10059 (skip inverse SRHT). They
  stack cleanly: 10059 deletes the QJL-disabled inverse-SRHT path, and
  10060 makes the remaining buffers reusable. Land in either order.
- This task is also independent of task 10061 (bytewise pack). 10061
  reduces `pack_mse_indices` cost; 10060 reduces allocator cost.

## Out of scope

- Making `EncodedTq` borrow scratch-owned buffers (would require an
  API break on the return type).
- Adding a bulk encode SQL function. That's task 10063.
- Threading scratch through any caller other than the new
  `encode_with_scratch` itself plus the `encode` wrapper.
- Microbenchmark infrastructure beyond a single inline `Instant`-based
  helper.

## Validate

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --no-default-features --features pg17 quant::prod
cargo test --no-default-features --features pg17 encode_with_scratch
```

All existing tests in `src/quant/prod.rs#tests` must pass unchanged.
The new `encode_with_scratch_matches_encode` test must pass on both the
QJL-disabled and QJL-enabled paths. Encoded payloads must be bit-exact
between the two entry points.

Branch from current upstream main. Push branch for review.
