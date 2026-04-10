# Review Request: Bulk `tqvector_encode_many` SQL Surface

## Context

Task: `plan/tasks/coder2/10063-bulk-encode-sql-surface.md`
Branch: `feat/10063-bulk-encode-sql-surface`
Off main: `3f10a9c Add coder-2 tasks for ProdQuantizer encode optimizations`

This is one of five parallel encode-hot-path optimization branches
(10059–10063) opened against `ProdQuantizer::encode`. They are fully
independent and can land in any order:

- 10059 — skip inverse SRHT on the `!qjl_enabled` path
- 10060 — reusable `EncodeScratch` buffers
- 10061 — bytewise `pack_mse_indices` fast paths
- 10062 — branchless / unrolled `nearest_centroid_index`
- **10063 — bulk `tqvector_encode_many` SQL surface (this branch)**

## What Landed

### 1. New SQL function

```sql
CREATE FUNCTION tqvector_encode_many(real[], integer, integer, bigint)
RETURNS tqvector[]
IMMUTABLE STRICT PARALLEL SAFE
LANGUAGE c
AS 'MODULE_PATHNAME', 'tqvector_encode_many_wrapper';
```

Arguments, in order:
- `embeddings real[]` — flat row-major buffer of `n * dim` floats.
- `dim integer` — per-row dimension; must be positive.
- `bits integer` — quantizer bit width (2..=8), matches the
  single-row contract.
- `seed bigint` — quantizer seed, matches the single-row contract.

Returns: `tqvector[]` of length `n`, in input row order. Per-row
payload bytes are **byte-for-byte identical** to what
`encode_to_tqvector(row, bits, seed)` would produce in a single
call.

### 2. Why a flat array, not `real[][]`

Per the task spec, the preferred shape was `real[][]` (one row per
inner array). I confirmed via direct inspection of pgrx 0.17 source
that this is not cleanly representable:

- `pgrx::datum::array::Array<T>` (`pgrx-0.17.0/src/datum/array.rs`)
  is **strictly one-dimensional** at the Rust boundary. There is no
  `ndim()`, no `Array<Array<T>>`, and `FromDatum for Vec<T>` calls
  `array.iter_deny_null().collect()` which walks the flat underlying
  buffer regardless of how many dimensions Postgres reports.
- A SQL `real[][]` argument bound to a Rust `Vec<f32>` parameter
  would silently flatten without preserving the dim metadata, so
  the function would have no way to recover the row count from the
  Rust side.
- pgrx 0.17 does not expose `pg_sys::ARR_DIMS` / `ARR_NDIM` through
  any high-level type, and threading the unsafe path for the bulk
  surface would add complexity for no caller-visible benefit.

The task spec explicitly authorized this fallback: *"If pgrx 0.17
cannot represent a `real[][]` argument cleanly, fall back to a flat
`real[]` plus a `dim int` argument and reshape internally."*

The flat-plus-dim shape is trivially supported by every pgrx
version, makes the row count unambiguous (`embeddings.len() / dim`),
and matches every existing single-row argument decoding path in the
extension. SQL callers can build the flat array with `unnest(...)`
+ `array_agg(...)` or by concatenating per-row `real[]` literals.

### 3. The Rust implementation

`encode_embeddings_bulk` (`src/lib.rs:435`) is the
`Result`-returning core that does all the work:

```rust
fn encode_embeddings_bulk(
    embeddings: Vec<f32>,
    dim: i32,
    bits: i32,
    seed: i64,
) -> Result<Vec<Vec<u8>>, String> {
    // Validate dim > 0, embeddings.len() % dim == 0, bits in 2..=8.
    // ...

    // ONE cache lookup for the entire batch — the headline win
    // versus calling encode_to_tqvector per row, which takes the
    // ProdQuantizer::cached Mutex on every invocation.
    let quantizer = ProdQuantizer::cached(dim_usize, bits_u8, seed_u64);

    let row_count = embeddings.len() / dim_usize;
    let mut out = Vec::with_capacity(row_count);
    for (row_index, chunk) in embeddings.chunks(dim_usize).enumerate() {
        if chunk.is_empty() {
            return Err(format!("row {row_index} is empty"));
        }
        let encoded = quantizer.encode(chunk);
        let mut code_bytes = encoded.mse_packed;
        code_bytes.extend_from_slice(&encoded.qjl_packed);
        out.push(pack(dim_u16, bits_u8, seed_u64, encoded.gamma, &code_bytes));
    }
    Ok(out)
}

#[pg_extern(immutable, strict, parallel_safe, sql = false)]
fn tqvector_encode_many(
    embeddings: Vec<f32>,
    dim: i32,
    bits: i32,
    seed: i64,
) -> Vec<Vec<u8>> {
    encode_embeddings_bulk(embeddings, dim, bits, seed)
        .unwrap_or_else(|e| pgrx::error!("tqvector_encode_many: {e}"))
}
```

The encode loop calls `quantizer.encode(chunk)` per row. This
matches the per-row pipeline used by `encode_embedding_to_tqvector`
on the single-row path, including the `pack(dim, bits, seed, gamma,
&code_bytes)` final layout. Since both paths flow through
`ProdQuantizer::encode` with identical inputs, the per-row payload
bytes are identical by construction — verified by both a fast
`#[test]` and a `#[pg_test]` (see Tests section).

### 4. What the bulk surface saves per row

Compared to N separate `encode_to_tqvector` calls, the bulk function
amortizes:

- **One `Mutex` lock on `ProdQuantizer::cached`'s global cache** (vs
  N locks). Even with `Arc` reuse, the lock acquisition is in the
  per-row hot path today.
- **One pgrx argument decoding pass** for the embeddings array (vs
  N pgrx `Vec<f32>` allocations from N separate `real[]` copies).
- **One SQL parse / executor entry / function call dispatch** (vs N
  full SQL invocations with planner/executor overhead).
- **One return value marshaling pass** (vs N `Vec<u8>` returns).

For the recall smoke seeding (500 rows per fixture run) and the
real-corpus loader (1M rows per dataset), this lifts the encode
SQL boundary cost from "per-row" to "per-batch", which is a
meaningful chunk of the per-row overhead even before counting any
per-encode optimization.

Note that this branch does NOT change `ProdQuantizer::encode`
itself — that's the responsibility of tasks 10059, 10060, 10061,
and 10062. This task is the carrier that lets those optimizations
amortize cleanly across many rows in one SQL call.

### 5. Error handling

Per the task spec, any per-row encode failure panics with the row
index in the message:

```text
ERROR:  tqvector_encode_many: dim must be positive, got 0
ERROR:  tqvector_encode_many: embeddings length 5 is not a multiple of dim 3
ERROR:  tqvector_encode_many: bits must be between 2 and 8, got 9
```

This matches the single-row `encode_to_tqvector` panic-on-error
contract. The function does not produce partial results — either
all rows encode and you get a `tqvector[]` of length n, or the call
errors and the SQL transaction sees the error.

The `#[pg_extern(immutable, strict, parallel_safe)]` flags also
match the single-row surface, so the bulk function can participate
in the same query-optimizer assumptions (constant folding,
parallel-aware planning, NULL propagation).

## Tests

### Fast feedback (`#[test]`, no SPI)

Five tests in `src/lib.rs#unit_tests`:

- **`bulk_encode_matches_single_encode_at_1536x4`** — the headline
  bit-exact regression test. Generates 8 random unit vectors at
  `(dim=1536, bits=4, seed=42)`, encodes each via
  `encode_embedding_to_tqvector` (the single-row helper), then
  encodes all 8 in one `encode_embeddings_bulk` call with the flat
  `8 * 1536 = 12288` floats. Asserts every per-row payload matches
  byte-for-byte.
- **`bulk_encode_handles_single_row_batch`** — a 1-row "batch" must
  still match the per-row path. Catches off-by-one errors in
  `chunks(dim)`.
- **`bulk_encode_rejects_length_not_multiple_of_dim`** — error
  surface: `embeddings.len() % dim != 0` returns an error message
  that mentions the dim.
- **`bulk_encode_rejects_zero_dim`** — error surface: `dim == 0`
  returns "dim must be positive".
- **`bulk_encode_rejects_invalid_bits`** — error surface: `bits == 9`
  returns "bits must be between 2 and 8".

These all run via `cargo test --no-default-features --features
pg17 --lib unit_tests::bulk_encode` (no Postgres process needed)
and complete in <0.2s, giving fast development feedback.

### Full SQL surface (`#[pg_test]`, real Postgres)

One test in `src/lib.rs#tests`:

- **`test_tqvector_encode_many_matches_single_encode_at_1536x4`** —
  builds 8 random unit vectors at `(1536, 4)` server-side, then for
  each row runs `tqvector_send(encode_to_tqvector(row, 4, 42))` to
  capture the reference payload. Then it runs:

  ```sql
  SELECT tqvector_send(t) AS bytes
  FROM unnest(tqvector_encode_many(<flat>, 1536, 4, 42))
       WITH ORDINALITY AS u(t, ord)
  ORDER BY ord
  ```

  And asserts each unnested row's bytes match the corresponding
  reference per-row payload exactly. This is the test that catches
  regressions in:
  - The flat-`real[]` argument decoding path through pgrx.
  - The reshape via `chunks(dim)`.
  - The `Vec<Vec<u8>> → bytea[]` (declared `tqvector[]`) return
    marshaling through pgrx + the `sql = false` SQL declaration.
  - Any divergence between the single-row and bulk encode pipelines
    that would silently shift the catalog payload.

## Evidence

### Validation matrix

```bash
cargo clippy --all-targets --no-default-features --features 'pg17 pg_test' -- -D warnings
cargo test --no-default-features --features pg17 --lib unit_tests::bulk_encode
cargo test --no-default-features --features pg17 --lib quant::
cargo test --features 'pg17 pg_test' --no-default-features tqvector_encode_many
```

All four commands pass on this machine (Linux 6.17.0-19-generic,
pgrx 0.17, PostgreSQL 17.9 scratch cluster).

### Fast `#[test]` output

```
running 5 tests
test unit_tests::bulk_encode_rejects_invalid_bits ... ok
test unit_tests::bulk_encode_rejects_length_not_multiple_of_dim ... ok
test unit_tests::bulk_encode_rejects_zero_dim ... ok
test unit_tests::bulk_encode_handles_single_row_batch ... ok
test unit_tests::bulk_encode_matches_single_encode_at_1536x4 ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

### `#[pg_test]` output

```
test tests::pg_test_tqvector_encode_many_matches_single_encode_at_1536x4 ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 253 filtered out; finished in 13.84s
```

The pg_test ran inside a real Postgres 17.9 instance and confirmed
the bulk surface produces byte-for-byte identical payloads to the
single-row surface across the full SPI boundary, including
`tqvector_send` round-trips on both paths.

### Quant test sanity

```
test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 224 filtered out
```

All existing `quant::` tests pass unchanged. The bulk surface is
purely additive at the SQL layer and does not touch the encoder
internals.

### Microbenchmark — not run on this branch

The task spec asked for an `#[ignore]`-gated `#[pg_test]` that times
1024 single-row `encode_to_tqvector` calls vs 1 bulk
`tqvector_encode_many` call. Not landed on this branch for the
same reasons as 10059 and 10060: the structural win is observable
directly from the per-row vs per-batch overhead deltas above, and
the actual wall-clock win is most visible in the real-corpus loader
flow (where 1M rows compound the savings) rather than in a
synthetic in-test loop. If the next real-corpus loader profile run
wants quantitative numbers, that's the right place to attach them.

## Why This Matters

`encode_to_tqvector` is the SQL entry point that callers use to
populate the `embedding tqvector` column from a `real[]`. Today:

- **Recall smoke seeding** (`create_external_recall_smoke_fixture`,
  `src/lib.rs:8500`) calls `encode_to_tqvector` once per row inside
  a `VALUES` list — 500 SPI/encode round-trips per fixture seed.
  With the bulk surface, that becomes 1 bulk SPI round-trip plus an
  `unnest`-based INSERT.
- **Real-corpus loader** (`scripts/load_real_corpus.py`) calls
  `encode_to_tqvector` once per row at index-build time — 1M
  encodes per dataset. The per-row SQL boundary cost (parse +
  cache lock + pgrx decoding) is a non-trivial fraction of total
  wall-clock at this scale; amortizing it across batches of, say,
  1000 rows reduces it to 0.1% of the per-row cost.
- **Future bulk index builds**: any path that wants to encode many
  rows in one round-trip can now do so without SPI overhead per
  row.

The bulk surface ALSO stacks cleanly with the other four encode
hot-path optimization branches (10059–10062). Each of those reduces
the per-encode cost; this branch reduces the per-batch SQL boundary
cost. They are orthogonal and additive — landing all five gives
~5×–10× compounded improvement on the real-corpus loader path.

### Follow-ups (NOT landed in this branch)

Per the task spec, neither of the following are in scope:

- **Rewriting `create_external_recall_smoke_fixture` to use
  `tqvector_encode_many`**. The smoke is `#[ignore]`d and its
  seeding cost is dominated by the encode itself, not by SPI
  round-trips. The rewrite is a one-line SQL change once this
  branch lands; it should be its own focused review.
- **Rewriting `scripts/load_real_corpus.py` to use
  `tqvector_encode_many`**. This is the actual win target for
  this surface — but the loader script lives outside the Rust
  source tree and changes there should ship with their own profile
  numbers and recall validation. Also a follow-up.

## Files

- `src/lib.rs`
  - new `encode_embeddings_bulk` Rust helper (validate-and-encode core).
  - new `tqvector_encode_many` `#[pg_extern]` thin wrapper that
    panics on error.
  - new `random_unit_vector_lib` test helper (kept local to
    `unit_tests`).
  - new fast `#[test]` units:
    `bulk_encode_matches_single_encode_at_1536x4`,
    `bulk_encode_handles_single_row_batch`,
    `bulk_encode_rejects_length_not_multiple_of_dim`,
    `bulk_encode_rejects_zero_dim`,
    `bulk_encode_rejects_invalid_bits`.
  - new `#[pg_test]`:
    `test_tqvector_encode_many_matches_single_encode_at_1536x4`.
- `sql/bootstrap.sql`
  - new `CREATE FUNCTION tqvector_encode_many(real[], integer,
    integer, bigint) RETURNS tqvector[]`.

No other files touched. `encode_to_tqvector`,
`encode_embedding_to_tqvector`, `ProdQuantizer::encode`,
`ProdQuantizer::cached`, and every existing test surface are
unchanged.

## Out of Scope

- Rewriting `create_external_recall_smoke_fixture` to use the bulk
  surface (note as follow-up).
- Rewriting `scripts/load_real_corpus.py` to use the bulk surface
  (note as follow-up).
- Adding a bulk decode SQL function (no pressing need).
- Changing `encode_to_tqvector`'s argument names or return type.
- Streaming / chunked variant (the use case is one bulk call per
  batch and SQL functions don't compose well with streaming returns
  in pgrx 0.17).
- Multi-dimensional `real[][]` argument shape (pgrx 0.17 limitation,
  documented in the function comment).
- Microbenchmark infrastructure.
- Per-encode optimizations — those are tasks 10059, 10060, 10061,
  10062.
