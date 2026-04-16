# Review Request: C1 ADR-030 V2 Heap-F32 Rerank Cost Profile

## Context

Packet `371` proved that survivor-only heap-f32 rerank can lift the isolated
grouped-only `m=16` lane well above the quantized rerank ceiling:

- quantized isolated grouped `m=16`:
  - `0.938 @ ef=64`
  - `0.940 @ ef=128`
  - `0.946 @ ef=320`
- heap-f32 rerank with `window=50`:
  - `0.968 @ ef=64`
  - `0.972 @ ef=128`
  - `0.978 @ ef=320`

But the same packet also showed that the SQL operating point stayed worse than
pgvector:

- heap-f32 verified SQL means:
  - `3.738 ms @ ef=64`
  - `4.385 ms @ ef=128`
  - `7.212 ms @ ef=320`

So the remaining question was no longer "can heap-f32 rerank move recall?" It
was "what part of heap-f32 rerank is actually expensive?"

## Problem

Before spending more time on the heap-f32 lane, the branch needed a direct
answer to three narrower questions:

1. how much rerank work the live `LIMIT 10` grouped query is actually doing
2. how much of heap-f32 rerank cost is heap fetch versus `real[]` decode
   versus fp32 dot-product work
3. whether the next optimization target should be heap access or payload shape

## Planned Slice

One instrumentation batch.

1. add grouped-rerank-specific debug counters for:
   - quantized rerank score calls and elapsed time
   - heap-f32 rerank element calls and elapsed time
   - heap row fetch count plus fetch/decode/dot timing splits
2. expose a narrow `tests.tqhnsw_debug_grouped_rerank_profile(...)` helper for
   grouped scans
3. make that helper limit-aware so it profiles the actual `LIMIT 10` query
   shape instead of exhausting the whole scan
4. prove in pg tests that:
   - quantized mode only increments quantized rerank counters
   - heap-f32 mode only increments heap rerank counters
5. rerun the isolated grouped-only `m=16` lane in both:
   - `binary + quantized`
   - `binary + heap_f32`
   at `window=50` and `ef=64/128/320`

## Implementation

### Code

This packet does not change the production search policy. It only adds a
debug/profile surface under `test` / `pg_test`.

- `src/am/scan.rs`
  - extends `ScanDebugProfile` with grouped rerank counters
  - records quantized rerank comparison calls and elapsed time
  - records heap-f32 rerank element calls and elapsed time
  - splits heap-f32 rerank time into:
    - heap fetch
    - `real[]` decode
    - fp32 dot product
- `src/am/scan_debug.rs`
  - adds `debug_grouped_rerank_profile(index_oid, query, limit_count)`
  - the final helper stops after `limit_count` emitted tuples so it matches the
    `LIMIT 10` query shape used by the runtime lane
- `src/am/mod.rs`
  - re-exports the new debug helper
- `src/lib.rs`
  - adds `tests.tqhnsw_debug_grouped_rerank_profile(...)`
  - adds pg tests for quantized-only and heap-only counter behavior
- `scripts/sql/refresh_adr030_scratch_debug_helpers.sql`
  - registers the scratch wrapper for the new helper

### Scratch setup

I installed the updated pg17 `pg_test` build and refreshed the wrappers with:

- `./scripts/install_adr030_pg17_pg_test.sh`
- `./scripts/refresh_adr030_scratch_debug_helpers.sh`

Measurement runs used:

- `./scripts/restart_adr030_scratch.sh --window 50 --grouped-score-mode binary --rerank-mode quantized`
- `./scripts/restart_adr030_scratch.sh --window 50 --grouped-score-mode binary --rerank-mode heap_f32`

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

All profile measurements below use the same isolated grouped-only `m=16` lane
from packets `370` and `371`:

- corpus table: `scratch_tqhnsw_real_50k_grouped_m16only_corpus`
- index: `scratch_tqhnsw_real_50k_grouped_m16only_idx`
- queries: `tqhnsw_real_50k_queries_50`
- grouped score mode: `binary`
- grouped live rerank window: `50`
- profile limit: `10`

### 1. The live `LIMIT 10` lane reranks about 60 survivor rows/query

Quantized rerank profile:

| ef_search | mean total us | emitted rows | quantized rerank calls | quantized rerank us |
|----------:|--------------:|-------------:|-----------------------:|--------------------:|
| `64`  | `1858.9` | `10.0` | `60.0` | `68.7` |
| `128` | `2379.4` | `10.0` | `60.0` | `82.6` |
| `320` | `4581.8` | `10.0` | `60.0` | `75.3` |

Heap-f32 rerank profile:

| ef_search | mean total us | emitted rows | heap rerank calls | heap rows fetched |
|----------:|--------------:|-------------:|------------------:|------------------:|
| `64`  | `4374.6` | `10.0` | `60.0` | `60.0` |
| `128` | `4640.7` | `10.0` | `60.0` | `60.0` |
| `320` | `7302.6` | `10.0` | `60.0` | `60.0` |

So the live `LIMIT 10` grouped lane is not reranking "everything." It is
reranking about `60` survivor rows/query on this configuration.

### 2. Quantized rerank work is basically free

Quantized grouped rerank time stays around `0.07 .. 0.08 ms/query`:

- `68.7 us @ ef=64`
- `82.6 us @ ef=128`
- `75.3 us @ ef=320`

That is far too small to explain packet `371`'s SQL gap.

### 3. Heap-f32 rerank adds about `2.3 .. 2.7 ms/query`

Comparing mean profile totals:

| ef_search | quantized total us | heap-f32 total us | added heap-f32 us |
|----------:|-------------------:|------------------:|------------------:|
| `64`  | `1858.9` | `4374.6` | `2515.7` |
| `128` | `2379.4` | `4640.7` | `2261.3` |
| `320` | `4581.8` | `7302.6` | `2720.8` |

So heap-f32 rerank is paying a real extra cost, but the extra cost is a narrow
band rather than something exploding with `ef_search`.

### 4. The extra cost is overwhelmingly `real[]` decode, not heap fetch

Heap-f32 rerank breakdown:

| ef_search | heap score us | heap fetch us | heap decode us | heap dot us |
|----------:|--------------:|--------------:|---------------:|------------:|
| `64`  | `2577.9` | `137.0` | `2308.6` | `61.4` |
| `128` | `2184.7` | `42.4`  | `2011.6` | `64.4` |
| `320` | `2240.3` | `37.5`  | `2065.4` | `65.6` |

Two things are clear:

- heap fetch itself is small: only `37.5 .. 137.0 us/query`
- fp32 dot-product work is also small: only `61.4 .. 65.6 us/query`

Nearly all of the added cost sits in decoding heap `real[]` values into Rust
`Vec<f32>` buffers.

### 5. Materialization tracks the heap rerank cost

The grouped materialization timer also shifts by about the same amount:

| ef_search | quantized materialize us | heap-f32 materialize us |
|----------:|-------------------------:|------------------------:|
| `64`  | `73.9` | `2592.0` |
| `128` | `88.3` | `2198.2` |
| `320` | `83.6` | `2256.0` |

That matches the interpretation above: the rerank penalty is being paid in the
window materialization / survivor scoring tier, not in traversal.

## Interpretation

### 1. Heap I/O is not the blocker on this lane

The naive suspicion was that survivor heap access itself might be too expensive.
The profile says otherwise.

At `ef=128`, the mean cost split is:

- heap fetch: `42.4 us`
- heap decode: `2011.6 us`
- dot product: `64.4 us`

So the branch should not spend the next batch on heap fetch micro-optimizations.

### 2. The expensive part is decoding raw heap `real[]`

The current heap-f32 design pays most of its penalty turning Postgres array
datums into owned `Vec<f32>` buffers.

That makes the next design implication pretty direct:

- if we want a higher-recall rerank tier without giving up the latency story,
  the better target is a higher-fidelity payload stored in the index
- not repeated heap `real[]` decode on survivor rows

### 3. Heap-f32 rerank is a feasibility proof, not a good operating point

Packet `371` already showed that heap-f32 rerank can move recall materially.
This packet shows why that lever does not look attractive as the next product
lane:

- it is not expensive because of traversal
- it is not expensive because of heap page access
- it is expensive because of payload decode shape

So the heap-f32 rerank path has done its job:

- it proved the recall headroom exists
- it localized the cost to heap payload decode

## Outcome

This packet does not change the grouped runtime policy.

It narrows the decision:

- keep the current quantized rerank lane as the real low-latency grouped mode
- treat heap-f32 rerank as an upper-bound / feasibility experiment
- if we want to push recall without giving up the low-latency corner, the next
  meaningful design is an in-index higher-fidelity rerank payload rather than
  more work on heap-f32 survivor fetch
