# Review Request: C1 ADR-030 V2 Zero-Copy Heap Array Decode

## Context

Packet `372` instrumented the heap-f32 rerank tier and localized its cost on
the isolated grouped-only `m=16` lane:

- heap fetch itself was small
- fp32 dot-product work was small
- the dominant cost was decoding heap `real[]` values via
  `Vec::<f32>::from_polymorphic_datum(...)`

Reviewer follow-up suggested trying the obvious next lever before giving up on
heap-f32 rerank entirely:

- keep the same heap-f32 survivor rerank semantics
- replace the pgrx array decode with a flat, validated array view
- score directly on a borrowed `&[f32]` without allocating a new `Vec<f32>`

That is the entire purpose of this packet.

## Problem

The branch needed to answer a narrower question than packet `372`:

1. does removing the `real[] -> Vec<f32>` allocation/conversion path materially
   reduce heap-f32 rerank cost
2. if it does, is the result enough to change the SQL operating point
3. if it does not, should the branch stop pushing heap-f32 rerank and go back
   to the in-index payload question

## Planned Slice

One code + measurement batch.

1. replace the heap-f32 rerank `real[]` decode with a single guarded helper
   that:
   - detoasts the array to aligned storage
   - validates one-dimensional `real[]` shape with no null elements
   - computes a flat data pointer and borrowed `&[f32]`
   - frees any detoasted copy on drop
2. leave the exact fp32 score math unchanged
3. rerun packet `372`'s limit-aware grouped rerank profile on the same isolated
   lane
4. rerun packet `371`'s verified SQL latency lane on the same isolated lane

## Implementation

### Code

- `src/am/scan.rs`
  - adds `FlatFloat4ArrayRef`, a small owned/borrowed wrapper around a detoasted
    PostgreSQL `ArrayType`
  - validates:
    - datum is present
    - `ndim == 1`
    - `elemtype == FLOAT4OID`
    - no null array elements
    - data pointer alignment for `f32`
  - computes the flat array data pointer directly from the `ArrayType` header
    instead of routing through `Vec::<f32>::from_polymorphic_datum(...)`
  - heap-f32 rerank now scores `negative_inner_product(query, source.as_slice())`
    directly on the borrowed slice

There are no SQL surface changes and no search-policy changes in this packet.

### Behavioral scope

This does not change rerank semantics.

It only changes how the heap `real[]` source vector is decoded before the exact
fp32 dot product is computed.

## Validation

Local checkpoint commands:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `./scripts/install_adr030_pg17_pg_test.sh`
- `./scripts/refresh_adr030_scratch_debug_helpers.sh`

Required checkpoint commands that still fail on this workstation at the same
known PostgreSQL/pgrx linker layer:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Both failures are unchanged from prior packets and still stop at unresolved
symbols such as `CurrentMemoryContext`, `PG_exception_stack`, and `errstart`
before test execution.

## Measurements

### Lane under test

All measurements below use the same isolated grouped-only `m=16` lane as
packets `371` and `372`:

- corpus table: `scratch_tqhnsw_real_50k_grouped_m16only_corpus`
- index: `scratch_tqhnsw_real_50k_grouped_m16only_idx`
- queries: `tqhnsw_real_50k_queries_50`
- grouped score mode: `binary`
- grouped live rerank window: `50`

### 1. The zero-copy decode helps, but decode is still the cost center

Packet `372` warmed heap-f32 rerank profile:

| ef_search | total us | heap score us | heap fetch us | heap decode us | heap dot us |
|----------:|---------:|--------------:|--------------:|---------------:|------------:|
| `64`  | `4374.6` | `2577.9` | `137.0` | `2308.6` | `61.4` |
| `128` | `4640.7` | `2184.7` | `42.4`  | `2011.6` | `64.4` |
| `320` | `7302.6` | `2240.3` | `37.5`  | `2065.4` | `65.6` |

This packet's warmed rerun after the custom flat-array decode:

| ef_search | total us | heap score us | heap fetch us | heap decode us | heap dot us |
|----------:|---------:|--------------:|--------------:|---------------:|------------:|
| `64`  | `3050.7` | `1811.6` | `21.5` | `1663.2` | `61.2` |
| `128` | `3777.7` | `1788.0` | `15.7` | `1645.3` | `61.3` |
| `320` | `5882.8` | `1769.7` | `12.4` | `1627.6` | `62.9` |

So the zero-copy-ish path is real:

- decode dropped by about `366 .. 645 us/query`
- total profile time dropped by about `863 .. 1420 us/query`

But the shape did not change:

- heap fetch is still tiny
- fp32 dot work is still tiny
- decode is still the dominant bucket at about `1.63 .. 1.66 ms/query`

### 2. The SQL lane improved, but not enough to change the verdict

Packet `371` heap-f32 verified SQL means:

| ef_search | mean ms |
|----------:|--------:|
| `64`  | `3.738` |
| `128` | `4.385` |
| `320` | `7.212` |

This packet's two verified SQL reruns:

Run 1:

| ef_search | mean ms |
|----------:|--------:|
| `64`  | `3.328` |
| `128` | `4.181` |
| `320` | `8.183` |

Run 2:

| ef_search | mean ms |
|----------:|--------:|
| `64`  | `3.325` |
| `128` | `4.136` |
| `320` | `6.650` |

The lower-`ef` points improved by roughly:

- `~0.41 ms @ ef=64`
- `~0.20 .. 0.25 ms @ ef=128`

The `320` point is noisier, but the warmed rerun still improved by about
`0.56 ms` versus packet `371`.

### 3. This still does not beat pgvector on the same isolated lane

Packet `370`'s pgvector `m=16` matched-session SQL mean band on the same
`50`-query subset was:

- `1.277 .. 1.602 ms @ ef=40`
- `1.610 .. 1.789 ms @ ef=64`
- `2.942 .. 3.000 ms @ ef=128`
- `6.432 .. 6.540 ms @ ef=320`

So even after the zero-copy decode improvement, heap-f32 rerank still remains
behind pgvector on this lane:

- tqvector heap-f32 `ef=64`: `~3.33 ms`
- tqvector heap-f32 `ef=128`: `~4.14 .. 4.18 ms`
- tqvector heap-f32 `ef=320`: `~6.65 .. 8.18 ms`

## Interpretation

### 1. The reviewer was right that pgrx decode overhead was real

This was not a dead end.

Removing `Vec::<f32>::from_polymorphic_datum(...)` did recover meaningful time
from the heap-f32 rerank tier.

So packet `372`'s conclusion should be refined:

- it was not heap page access that made heap-f32 expensive
- and it was not purely inevitable fp32 work
- part of the cost really was the high-level array decode path

### 2. But the optimization is not enough

Even after this change:

- decode remains the largest heap-f32 bucket
- heap-f32 rerank still costs about `1.77 .. 1.81 ms/query` in the rerank tier
- the SQL operating point still does not beat pgvector

So this optimization improves the heap-f32 lane, but it does not rescue it.

### 3. The branch should not spend many more batches here

This was the best obvious heap-side optimization, and it helped.
But the result is still not attractive enough to justify a longer heap-rerank
subproject.

That points back to the same next design seam, now with stronger evidence:

- if we want higher recall without giving up the low-latency story,
  the more promising path is a higher-fidelity rerank payload stored in the
  index
- not more survivor heap rerank work

## Outcome

This packet keeps the custom flat-array decode because it is strictly better
than the old pgrx `Vec<f32>` conversion.

But it does not change the broader branch decision:

- quantized rerank remains the real low-latency grouped mode
- heap-f32 rerank remains a useful recall upper-bound experiment
- the next meaningful recall-vs-latency push should target in-index rerank
  payload fidelity, not additional heap-side rerank optimization
