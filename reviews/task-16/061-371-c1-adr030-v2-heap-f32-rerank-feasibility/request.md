# Review Request: C1 ADR-030 V2 Heap-F32 Rerank Feasibility

## Context

Packet `370` reconciled the isolated grouped-only `m=16` lane and showed the
current quantized payload stack plateauing at:

- direct isolated grouped tqvector `m=16`: `0.940 @ ef=128`, `0.946 @ ef=320`,
  `0.950 @ ef=512..1000`
- matched-session SQL means: `2.068..2.265 ms @ ef=128`,
  `4.666..4.964 ms @ ef=320`
- pgvector `m=16` measured floor on the same `50`-query subset:
  `0.986 @ ef=40`, `0.998 @ ef=128`

Reviewer-2 on packet `369` argued that this was a ceiling of the current
payload stack, not of the system, and pointed at a specific next lever:

- keep binary traversal
- widen the live grouped rerank window
- fetch raw `real[]` source vectors from the heap for final survivors
- rerank those survivors by exact fp32 inner product

That is the lever this packet tests.

## Problem

The branch needed to answer three questions before treating packet `369`'s
latency-first framing as durable:

1. can survivor-only heap-f32 rerank move the isolated grouped `m=16` recall
   curve materially above the quantized payload ceiling from packet `370`
2. if it can, what survivor window is required
3. does the resulting SQL operating point beat pgvector or merely prove that
   the quantized rerank ceiling was not fundamental

## Planned Slice

One code + measurement batch.

1. add a gated grouped rerank mode that replaces the live window's comparison
   scorer with exact heap-f32 inner product on buffered survivors
2. keep traversal, buffering, and the grouped live window structure unchanged
3. expose the rerank mode in the runtime-settings surface and scratch restart
   wrapper
4. prove on a pg fixture that:
   - the runtime gate is visible
   - invalid rerank env values are rejected
   - grouped live output/comparison scores match the raw heap f32 score when
     the mode is enabled
5. rerun the isolated grouped-only `m=16` lane on the scratch cluster with
   `binary + heap_f32` and survivor windows `10/20/30/50/64`
6. run verified SQL latency on the best promising window

## Implementation

### Code

The grouped scan runtime now accepts:

- `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_MODE=quantized|heap_f32`

When `heap_f32` is enabled on a grouped-v2 scan:

- `src/am/scan.rs`
  - resolves the rerank mode once per `amrescan`
  - allocates a scan-local heap tuple slot plus source-column attnum when the
    mode is active
  - uses `table_tuple_fetch_row_version(...)` to fetch survivor heap rows by
    TID
  - decodes the raw `real[]` source vector and scores `-dot(query_f32,
    source_f32)`
  - ranks the live grouped window by that exact heap score
  - emits that exact heap score as the order-by score while preserving the
    approximate grouped score as the sidecar `approx_score`
- `src/lib.rs`
  - runtime-settings wrapper now exposes `grouped_scan_rerank_mode`
  - pg tests prove heap-rerank score correctness and invalid-env rejection
- `scripts/restart_adr030_scratch.sh`
  - new `--rerank-mode quantized|heap_f32`
- `scripts/sql/refresh_adr030_scratch_debug_helpers.sql`
  - scratch runtime-settings wrapper now includes the rerank mode column

### Scratch setup

I installed the updated pg17 `pg_test` build with:

- `./scripts/install_adr030_pg17_pg_test.sh`

and refreshed the scratch wrappers with:

- `./scripts/refresh_adr030_scratch_debug_helpers.sh`

## Validation

Local checkpoint commands:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `bash -n scripts/restart_adr030_scratch.sh`

Required checkpoint commands that still fail on this workstation at the same
known PostgreSQL/pgrx linker layer:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Both failures are unchanged from prior packets and still stop at unresolved
symbols such as `CurrentMemoryContext`, `PG_exception_stack`, and `errstart`
before test execution.

Scratch runtime settings during the measurement runs were verified via
`tests.tqhnsw_debug_adr030_runtime_settings()` and matched the expected:

- grouped build enabled = `true`
- grouped scan enabled = `true`
- grouped scan score mode = `binary`
- grouped scan rerank mode = `heap_f32`
- grouped exact traversal enabled = `false`

## Measurements

### Lane under test

All direct and SQL measurements below use the isolated grouped-only `m=16`
lane from packet `370`:

- corpus table: `scratch_tqhnsw_real_50k_grouped_m16only_corpus`
- index: `scratch_tqhnsw_real_50k_grouped_m16only_idx`
- queries: `tqhnsw_real_50k_queries_50`

### 1. Small survivor windows can be actively bad

Direct isolated grouped tqvector `m=16` recall at small windows:

| rerank window | ef_search | Recall@10 |
|--------------:|----------:|----------:|
| `10` | `64`  | `0.870` |
| `10` | `128` | `0.872` |
| `10` | `320` | `0.876` |
| `20` | `128` | `0.930` |

So reviewer-2's survivor-pool caveat is real: exact heap rerank on too-narrow a
window can underperform the quantized grouped baseline because the pool does not
capture enough of the true top-10.

### 2. Larger survivor windows materially lift recall

Direct isolated grouped tqvector `m=16` recall with wider heap rerank windows:

| rerank window | ef_search | Recall@10 |
|--------------:|----------:|----------:|
| `30` | `128` | `0.952` |
| `50` | `64`  | `0.968` |
| `50` | `128` | `0.972` |
| `50` | `320` | `0.978` |
| `64` | `128` | `0.974` |

Compared with packet `370`'s isolated quantized baseline:

| mode | ef_search | Recall@10 |
|------|----------:|----------:|
| quantized rerank | `64`  | `0.938` |
| heap-f32 rerank `window=50` | `64` | `0.968` |
| quantized rerank | `128` | `0.940` |
| heap-f32 rerank `window=50` | `128` | `0.972` |
| quantized rerank | `320` | `0.946` |
| heap-f32 rerank `window=50` | `320` | `0.978` |

So the reviewer's main thesis is confirmed:

- the quantized rerank ceiling from packet `370` was not fundamental
- survivor-only heap-f32 rerank can move the isolated grouped lane well above
  that ceiling

### 3. Verified SQL latency on the promising window

Verified matched-session SQL rerun on `window=50`:

| ef_search | p50 ms | p95 ms | mean ms |
|----------:|-------:|-------:|--------:|
| `64`  | `3.639` | `4.190` | `3.738` |
| `128` | `4.241` | `5.165` | `4.385` |
| `320` | `7.089` | `8.510` | `7.212` |

For comparison:

- packet `370` quantized isolated tqvector SQL mean band:
  - `ef=64`: `1.361 .. 1.486 ms`
  - `ef=128`: `2.068 .. 2.265 ms`
  - `ef=320`: `4.666 .. 4.964 ms`
- packet `370` pgvector SQL mean band:
  - `ef=40`: `1.277 .. 1.602 ms` with `Recall@10 = 0.986`
  - `ef=64`: `1.610 .. 1.789 ms`
  - `ef=128`: `2.942 .. 3.000 ms` with `Recall@10 = 0.998`
  - `ef=320`: `6.432 .. 6.540 ms` with `Recall@10 = 0.998`

## Interpretation

### 1. The reviewer found a real recall lever

Heap-f32 survivor rerank is not a no-op.

It lifts the isolated grouped-only `m=16` lane from:

- `0.940 @ ef=128` quantized

to:

- `0.972 @ ef=128` with `window=50`

and reaches:

- `0.978 @ ef=320` with `window=50`

So packet `369` should not be read as "the system topped out at `0.95`." It
topped out there only within the quantized rerank stack.

### 2. The cost curve is also real

The recall gain is expensive.

`window=50` raises SQL latency from roughly:

- `2.068..2.265 ms` to `4.385 ms` at `ef=128`
- `4.666..4.964 ms` to `7.212 ms` at `ef=320`

That is enough to erase the remaining product case on this lane.

At the measured points:

- tqvector heap-f32 `window=50`, `ef=128`:
  - `Recall@10 = 0.972`
  - SQL mean `4.385 ms`
- pgvector `ef=40`:
  - `Recall@10 = 0.986`
  - SQL mean band `1.277 .. 1.602 ms`
- pgvector `ef=128`:
  - `Recall@10 = 0.998`
  - SQL mean band `2.942 .. 3.000 ms`

and:

- tqvector heap-f32 `window=50`, `ef=320`:
  - `Recall@10 = 0.978`
  - SQL mean `7.212 ms`
- pgvector `ef=320`:
  - `Recall@10 = 0.998`
  - SQL mean band `6.432 .. 6.540 ms`

So on the current isolated `50k` / `m=16` / `50`-query lane:

- heap-f32 rerank proves the recall ceiling can move
- but it does **not** produce a better operating point than pgvector
- it also gives up the low-latency tqvector pocket that packet `370`
  identified

### 3. Current verdict

The heap-f32 rerank spike is valuable because it closes an architectural
question:

- yes, there is a path above the quantized payload ceiling

But the measured operating-point answer on this lane is still negative:

- better recall is available
- not at a latency that beats pgvector

So this batch changes the architectural diagnosis, not the product verdict.

## Next Step

Two reasonable next moves remain:

1. stop here and record heap-f32 rerank as a useful feasibility result, but not
   a shipping operating point on this lane
2. only continue if there is a concrete cost-reduction hypothesis for the heap
   rerank tier itself
   - e.g. reducing survivor fetch cost or changing the survivor population shape

What does **not** look justified now is more blind window sweeping. The
`10/20/30/50/64` sweep already shows the basic shape clearly:

- narrow windows fail on capture
- wide windows recover recall
- the recovered recall costs too much SQL time on this lane
