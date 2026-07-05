# Review Request: C1 ADR-030 V2 Scan-Only Bytea Rerank Source Experiment

## Context

Packet `373` proved that removing the high-level pgrx `real[] -> Vec<f32>` path
helped, but heap-f32 rerank still did not reach a good SQL operating point.

Reviewer follow-up asked a narrower question:

- if the remaining heap-rerank cost is still tied to PostgreSQL `real[]`
  handling, can a cheaper heap payload shape improve things further
- can we answer that without rewriting the build contract away from
  `build_source_column = 'source'`

This packet isolates exactly that experiment.

## Problem

There were two open questions:

1. does a scan-only alternate heap source type beat the current optimized
   `real[]` rerank path once we measure the full live rerank path
2. what exactly did "not full query impact" mean in the earlier discussion

The second point matters because a decoder-only microbench would miss:

- tuple-slot attribute extraction
- detoast behavior
- survivor-loop integration
- exact dot-product timing inside a live grouped scan

So the branch needed a live scan experiment, not just a standalone decoder
benchmark.

## Planned Slice

One isolated code + measurement batch on a side branch.

1. keep the existing grouped-v2 build contract unchanged
   - index build still reads `build_source_column = 'source'`
2. add a scan-only rerank source override
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_SOURCE_COLUMN`
3. allow grouped heap-f32 rerank to read either:
   - `real[]`
   - `bytea`
4. add a small helper to pack `real[]` into raw little-endian `f32` bytes
5. build a scratch copy that carries both:
   - `source real[]`
   - `source_raw bytea`
6. measure the same grouped index twice on the same heap copy:
   - once through `source`
   - once through `source_raw`

That answers the real rerank-path question without changing index layout or the
build path.

## Implementation

### Code

- `src/am/scan.rs`
  - adds `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_SOURCE_COLUMN`
  - grouped heap-f32 rerank now resolves its scan-time source column from:
    - the explicit override when present
    - otherwise the persisted `build_source_column`
  - validates the chosen source column as either:
    - `real[]`
    - `bytea`
  - adds `FlatFloat4ByteaRef`, a detoasted borrowed view over raw packed `f32`
    bytes
  - adds `FlatFloat4SourceRef` so the exact dot-product path can score either
    `real[]` or `bytea` through one seam
- `src/lib.rs`
  - adds `tests.tqhnsw_debug_pack_f32_bytea(real[]) -> bytea`
  - extends grouped runtime fixtures with an optional `source_raw bytea` column
  - adds grouped runtime coverage proving the bytea override emits the same
    exact scores as the original heap source
  - extends `tests.tqhnsw_debug_adr030_runtime_settings()` with
    `grouped_scan_rerank_source_column`
- `scripts/restart_adr030_scratch.sh`
  - adds `--rerank-source-column`
- `scripts/sql/refresh_adr030_scratch_debug_helpers.sql`
  - refreshes the new bytea pack helper wrapper
  - refreshes the runtime-settings wrapper signature

### Behavioral scope

This packet does **not** change:

- grouped traversal scoring
- grouped live-window policy
- index build layout
- recall semantics

It only changes which heap column the survivor rerank reads and how that heap
datum is decoded inside the live scan path.

## Validation

Local checkpoint commands:

- `cargo check --tests`
- `cargo check --tests --no-default-features --features 'pg17 pg_test'`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- `bash -n scripts/restart_adr030_scratch.sh scripts/refresh_adr030_scratch_debug_helpers.sh`
- `./scripts/install_adr030_pg17_pg_test.sh`
- `./scripts/refresh_adr030_scratch_debug_helpers.sh`

Required checkpoint commands that still fail on this workstation at the same
known PostgreSQL/pgrx linker layer:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`

Both failures are unchanged from prior packets and still stop at unresolved
symbols such as `CurrentMemoryContext`, `PG_exception_stack`, and `errstart`
before test execution.

Scratch runtime validation:

- restarted scratch with:
  - `--window 50`
  - `--grouped-score-mode binary`
  - `--rerank-mode heap_f32`
  - with and without `--rerank-source-column source_raw`
- installed refreshed pg-test wrappers
- built a new isolated scratch corpus copy:
  - `scratch_tqhnsw_real_50k_grouped_m16bytea_corpus`
  - `scratch_tqhnsw_real_50k_grouped_m16bytea_idx`
- verified planner use for every SQL timing cell through
  `scripts/bench_sql_latency_verified_scratch.sh`

## Measurements

### Lane under test

All numbers below use the same copied isolated grouped lane:

- corpus table: `scratch_tqhnsw_real_50k_grouped_m16bytea_corpus`
- index: `scratch_tqhnsw_real_50k_grouped_m16bytea_idx`
- queries: `tqhnsw_real_50k_queries_50`
- grouped score mode: `binary`
- grouped live rerank window: `50`
- top-k / profile limit: `10`

The only variable changed between the two runs is the survivor rerank source:

- baseline: `build_source_column` -> `source real[]`
- experiment: `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_RERANK_SOURCE_COLUMN=source_raw`

### 1. This measured real rerank-path impact, not just decode microbench cost

The profile surface used here is:

- `tests.tqhnsw_debug_grouped_rerank_profile(index_oid, query, limit_count)`

so each datapoint includes:

- heap row fetch
- datum extraction from the fetched slot
- detoast / decode work
- exact fp32 dot-product work
- the grouped survivor loop itself

What it does **not** include is unrelated planner or SQL launcher overhead.
That is why the packet also includes matched SQL timings below.

### 2. Packed `bytea` does not beat the optimized `real[]` path on the same live lane

Mean rerank profile on the same table + index:

Baseline `source real[]`:

| ef_search | total us | heap score us | heap fetch us | heap decode us | heap dot us |
|----------:|---------:|--------------:|--------------:|---------------:|------------:|
| `64`  | `3942.5` | `2109.6` | `125.3` | `1856.7` | `61.3` |
| `128` | `3731.5` | `1740.6` | `14.5`  | `1600.0` | `61.1` |
| `320` | `6123.7` | `1765.2` | `15.5`  | `1620.4` | `60.4` |

Override `source_raw bytea`:

| ef_search | total us | heap score us | heap fetch us | heap decode us | heap dot us |
|----------:|---------:|--------------:|--------------:|---------------:|------------:|
| `64`  | `3835.0` | `2189.5` | `112.6` | `1942.4` | `63.0` |
| `128` | `4409.5` | `1895.1` | `33.5`  | `1725.8` | `61.8` |
| `320` | `6859.5` | `1873.8` | `26.6`  | `1709.9` | `61.7` |

So on the actual rerank-path measurement:

- `bytea` decode is not lower than the optimized `real[]` decode
- heap rerank time is slightly worse on the bytea path
- the `real[]` zero-copy path remains the better internal scorer on this lane

### 3. SQL timings are effectively a wash, with no stable win for `bytea`

Matched-session verified SQL means on the same copied lane:

Baseline `source real[]`:

| ef_search | mean ms |
|----------:|--------:|
| `64`  | `3.303` |
| `128` | `4.310` |
| `320` | `6.885` |

Override `source_raw bytea`:

| ef_search | mean ms |
|----------:|--------:|
| `64`  | `3.333` |
| `128` | `4.469` |
| `320` | `6.782` |

This is not a real improvement curve:

- `bytea` is slightly slower at `64`
- `bytea` is clearly slower at `128`
- `bytea` is slightly faster at `320`

The effect is small and inconsistent, which is exactly what the internal
profile table already suggested.

## Interpretation

### 1. The earlier "not full query impact" caution was real

A standalone decoder microbench could have told us only:

- raw `bytea` view vs raw `real[]` view

This packet answers the stronger question:

- what happens when that alternate heap payload is used inside the real grouped
  survivor rerank path

That includes slot access, detoast, the live survivor loop, and end-to-end SQL
timing. So this packet is the full rerank-path answer without a full build
rewrite.

### 2. A different heap payload shape is not enough here

The main hope behind this spike was:

- strip away array metadata and generic `real[]` machinery
- keep the same exact score math
- recover a meaningful latency win

The result is negative:

- internal rerank cost does not improve
- SQL latency does not improve materially

So simply changing the heap payload from `real[]` to packed raw `f32` bytes is
not enough to move the operating point.

### 3. The branch should stop pushing heap-side exact rerank representations

At this point the heap-side sequence is:

1. high-level pgrx `real[] -> Vec<f32>`: too slow
2. flat zero-copy-ish `real[]`: meaningfully better, still not good enough
3. scan-only raw `bytea`: no material improvement over optimized `real[]`

That makes the next design choice clearer:

- if we want higher recall without surrendering the latency lane, the stronger
  remaining path is a higher-fidelity rerank payload stored in the index
- not more heap-source datatype experiments on the current survivor-rerank path

## Requested Review Focus

1. Does the scan-only override seam in `src/am/scan.rs` look appropriately
   narrow for an isolated datatype experiment?
2. Are the measurements sufficient to close the "try a cheaper heap
   representation" branch of investigation?
3. Given the negative result, is the next rational ADR030 lever now an in-index
   higher-fidelity rerank payload rather than another heap-source experiment?
