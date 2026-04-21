# Task: Bulk `encode_to_tqvector` SQL Surface

Motivation: `encode_to_tqvector` (`src/lib.rs:430`) is the SQL entry
point that callers use to populate the `embedding tqvector` column
from a `real[]`. Today it is single-row: each call pays pgrx
`Vec<f32>` argument decoding from a Postgres `real[]`, a `Mutex` lock
on the `ProdQuantizer` cache (`src/quant/prod.rs:41`), an `Arc::clone`,
the encode itself, and pgrx encoding the resulting `Vec<u8>` back to
Postgres. The recall smoke calls this 500 times per fixture seed. The
real-corpus index build path calls it ~1M times per dataset. None of
the per-row overhead is necessary — the cache lookup, the scratch
buffer (if task 10060 lands), and even the SQL parse can all be
amortized across many rows in a single call. A bulk encode SQL
function lets callers seed thousands of rows per round-trip and lets
the implementation share scratch state across the batch.
Priority: batch 3
Status: ready

## Prompt

Add a new `tqvector_encode_many(real[][], int, bigint) -> bytea[]` SQL
function that encodes an array of vectors in one call, sharing the
quantizer lookup and (if available) scratch buffers across all rows.
Keep the existing single-row `encode_to_tqvector` API working
unchanged.

### Step 1 — read the current single-row surface

Read, in order, before touching anything:

- `src/lib.rs:402` (`encode_embedding_to_tqvector`) — the Rust-only
  helper that wraps `ProdQuantizer::cached(...).encode(...)` and
  returns `Result<Vec<u8>, String>`.
- `src/lib.rs:430` (`encode_to_tqvector`) — the `#[pg_extern]` SQL
  surface that calls `encode_embedding_to_tqvector` and panics on
  error. Note the exact argument names and types: `embedding:
  Vec<f32>`, `bits: i32`, `seed: i64`. Note the return type: `Vec<u8>`
  (which pgrx encodes as Postgres `bytea`).
- `src/quant/prod.rs:70` (`ProdQuantizer::cached`) — the cache
  contract. One `Mutex` lock per call.
- `src/lib.rs:8498` (`create_external_recall_smoke_fixture`, modified
  in task 10057) — the smoke's seeding helper. This task makes that
  helper a candidate user of the new bulk surface, but **does not**
  rewrite it.
- pgrx 0.17 array support: pgrx exposes `Array<T>` for `T[]` arguments
  and `Vec<Vec<T>>` is *not* the right shape for `T[][]`. Read
  `~/.cargo/registry/src/index.crates.io-*/pgrx-0.17.0/src/datum/array.rs`
  for the actual `Array<Array<f32>>` or `Array<f32>` + dim hint
  pattern. **Confirm the right type signature before designing the
  function.** If pgrx 0.17 cannot represent a `real[][]` argument
  cleanly, fall back to a flat `real[]` plus a `dim int` argument
  and reshape internally.

### Step 2 — pick a callable shape

Two viable shapes:

- **A: `real[][]`** — one row per inner array. Cleanest for callers,
  but pgrx 0.17 may not expose multi-dim arrays as a clean Rust
  type. Verify with a one-line probe (`cargo build` against a stub
  function) before committing to this shape.
- **B: flat `real[]` + `dim int`** — caller passes the concatenated
  vector data and the dimension. Implementation reshapes internally
  via `chunks(dim)`. Less clean for SQL callers but trivially
  supported by every pgrx version.

Default to A if pgrx 0.17 supports it; fall back to B if not. Document
the choice in the function's `#[pg_extern]` doc string and in the
review packet. Either way, the function should also accept `bits:
i32` and `seed: i64` matching `encode_to_tqvector`'s contract.

Return shape: `bytea[]`, one entry per input vector, in the same
order. (Or `Vec<Vec<u8>>` on the Rust side.)

### Step 3 — implement with one cache lookup and one scratch

```rust
#[pg_extern]
fn tqvector_encode_many(
    embeddings: /* shape A or B from step 2 */,
    bits: i32,
    seed: i64,
) -> Vec<Vec<u8>> {
    let bits_u8 = u8::try_from(bits).expect("bits must fit in u8");
    let seed_u64 = seed as u64;

    // Determine dim from the first vector (shape A) or from the
    // explicit `dim` argument (shape B). Validate all subsequent
    // vectors share that dim.
    let dim: usize = /* ... */;
    let quantizer = ProdQuantizer::cached(dim, bits_u8, seed_u64);

    // If task 10060 has landed, build a scratch once and reuse it
    // across all rows. Otherwise just call quantizer.encode in a loop.
    let mut scratch_opt = /* Some(quantizer.new_scratch()) if 10060
                            landed, else None */;

    // Encode every input vector, returning the packed payload bytes.
    // Reuse the same encode pipeline as the single-row surface so the
    // output is byte-for-byte identical to encoding the same vectors
    // one at a time via encode_to_tqvector.
    /* ... */
}
```

The bulk function MUST produce, for input vector `v` at position `i`,
the **exact same `Vec<u8>` payload bytes** as
`encode_to_tqvector(v, bits, seed)` would produce in a single call.
There is no room for "almost the same" here — the output is a packed
binary payload whose bytes are committed to the catalog and downstream
recall scoring.

If task 10060 (scratch buffers) has landed, use
`encode_with_scratch(&v, &mut scratch)` for each row, keeping one
scratch buffer for the whole batch. Wrap the scratch in `Some(...)`
only if the cache hit returned a quantizer whose scratch the function
can size — in practice every quantizer can produce a scratch via
`new_scratch()`, so this is unconditional.

If task 10060 has NOT landed, just call `quantizer.encode(&v)` in a
loop. The bulk surface still wins on the cache lookup, the SQL parse,
and the pgrx argument-decoding overhead — which is a meaningful
fraction of the per-row cost.

### Step 4 — error handling

The single-row `encode_to_tqvector` panics on `Err(String)` from
`encode_embedding_to_tqvector`. The bulk surface should match: if any
single row fails (wrong dimension, NaN gamma, whatever), panic with a
message that includes the **batch index** of the offending row so the
caller can find it. Do not swallow the error and produce a partial
result — the SQL contract is "all or nothing".

If pgrx exposes a `Result<Vec<Vec<u8>>, ...>` shape that translates
to a SQL error properly, prefer that over a panic. Match whatever the
existing single-row surface does for consistency.

### Step 5 — wire the smoke fixture (optional, low priority)

`create_external_recall_smoke_fixture` (`src/lib.rs:8498`, after task
10057) currently builds a single batched INSERT that calls
`encode_to_tqvector` once per row inside the SQL `VALUES` list. That
is 500 SPI/encode round-trips per fixture seed. With the new bulk
surface, the helper *could* be rewritten to call
`tqvector_encode_many` once and then INSERT the resulting `bytea[]`
column unnested.

This rewrite is **optional and out of scope for this task**. The
smoke's seeding cost is dominated by the encode itself, not by SPI
round-trips, and the smoke is `#[ignore]`d. Note the rewrite as a
follow-up in the review packet, do not land it here.

The real win for `tqvector_encode_many` is on the *real-corpus*
index build path, where the savings compound over millions of rows.
That use case is the responsibility of the real-corpus loader surface
(`ecaz corpus load`), not the smoke fixture. Document the
use case in the review packet but do not modify the loader
either — that is its own task and its own review.

### Step 6 — bit-exact regression test

Add a `#[pg_test]` test in `src/lib.rs` (alongside the existing
`#[pg_test]` recall tests) that:

1. Generates 8 random unit vectors at `dim=1536`.
2. Calls `encode_to_tqvector(v, 4, 42)` for each one and collects
   the `Vec<u8>` payloads.
3. Calls `tqvector_encode_many(all_vectors, 4, 42)` once and collects
   the `Vec<Vec<u8>>` payloads.
4. Asserts each pair of payloads is byte-for-byte equal.

Plus a smaller `#[test]` in `src/quant/prod.rs#tests` that exercises
the underlying encode path (without going through pgrx SPI) to give
faster feedback during development:

```rust
#[test]
fn bulk_encode_matches_single_encode() {
    let quantizer = ProdQuantizer::new(1536, 4, 42);
    let vectors: Vec<Vec<f32>> = (0..8)
        .map(|seed| random_unit_vector(1536, seed))
        .collect();
    let single: Vec<EncodedTq> = vectors.iter()
        .map(|v| quantizer.encode(v))
        .collect();
    // Whatever the bulk Rust helper looks like:
    let bulk: Vec<EncodedTq> = bulk_encode(&quantizer, &vectors);
    for (s, b) in single.iter().zip(bulk.iter()) {
        assert_eq!(s.gamma.to_bits(), b.gamma.to_bits());
        assert_eq!(s.mse_packed, b.mse_packed);
        assert_eq!(s.qjl_packed, b.qjl_packed);
    }
}
```

### Step 7 — measure

Add an `#[ignore]`-gated `#[pg_test]` that times encoding 1024 random
1536-dim vectors via:

- 1024 separate calls to `encode_to_tqvector` (the current path).
- 1 call to `tqvector_encode_many` with the array of 1024 vectors.

Report wall-clock for both. Target: at least **2× speedup** on the
bulk path against the per-row path. If task 10060 has landed and
scratch reuse is wired in, the target is **3-5×**.

## Design notes

- The single-row `encode_to_tqvector` stays exactly as it is. Existing
  callers, existing tests, existing SQL surfaces are all unchanged.
  The bulk surface is purely additive.
- Do not change the cache contract in `ProdQuantizer::cached`. The
  bulk function still goes through the same cache; it just hits it
  once per batch instead of once per row.
- Do not assume task 10060 has landed. The implementation should work
  whether or not scratch buffers exist. Coordinate with the
  reviewer on the merge order — if 10060 lands first, this task gets
  to use the scratch path. If not, this task lands a slightly slower
  bulk path and a follow-up adds scratch reuse.
- Validate input dimensions early: every inner vector must share the
  same length, or the function panics with a clear "row N has
  dimension M, expected K" message. Do not silently truncate or pad.
- Do not introduce a streaming / chunked variant that yields
  partial results. SQL functions don't compose well with streaming
  return types in pgrx 0.17, and the use case (one bulk call per
  batch) does not need it.

## Out of scope

- Rewriting `create_external_recall_smoke_fixture` to use the bulk
  surface (note as follow-up only).
- Rewriting `ecaz corpus load` to use the bulk surface
  (also note as follow-up).
- Adding a bulk decode surface. There is no pressing need.
- Changing `encode_to_tqvector`'s argument names or return type.

## Validate

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --no-default-features --features pg17 bulk_encode
cargo test --features 'pg17 pg_test' --no-default-features tqvector_encode_many -- --nocapture
```

The bit-exact regression tests must pass: encoded payloads from the
bulk surface must equal encoded payloads from the single-row surface,
byte-for-byte, for every input. Existing tests in
`src/quant/prod.rs#tests` and the `#[pg_test]` recall tests in
`src/lib.rs` must all still pass unchanged.

Branch from current upstream main. Push branch for review.
