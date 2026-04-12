# Review Request: C1 Disable Unused Query Prep

## Context

Packet `264` prioritized the warm steady-state optimization backlog and called
out one cheap correctness/perf cleanup before the heavier SIMD scorer work:
when the `1536`-dim, `4`-bit path runs with QJL disabled, query preparation
still builds data that the hot scorer never reads.

Current warm `10K`, `m=8`, `ef_search=40`, `warm-after-prime3`, `per-cell`
still sits around `14ms`, far above the C1 target, so this slice is not meant
to close the gap by itself. The goal is to remove known dead work before
benchmarking larger scorer changes.

## Problem

In `src/quant/prod.rs`:

- `prepare_ip_query(...)` still builds `PreparedQuery.lut` whenever
  `mse_bits != 3`
- the no-QJL `4`-bit scorer
  `score_ip_from_split_parts_no_qjl_4bit(...)` never reads that LUT
- `ProdQuantizer::new(...)` still materializes `qjl_signs` even for quantizers
  where `qjl_enabled(dim, bits)` is false

That means the no-QJL `4`-bit path still pays for query-prep/state that is
dead on arrival.

## Implementation

Completed work in `src/quant/prod.rs`:

1. `ProdQuantizer::new(...)` now leaves `qjl_signs` empty when
   `qjl_enabled(dim, bits)` is false instead of materializing a dead sign
   vector for the tiled `1536x4-bit` production path.
2. `prepare_ip_query(...)` now uses the same gate, so the no-QJL `4`-bit path
   skips both QJL projection work and LUT construction.
3. The scalar fallback now tolerates an intentionally empty `PreparedQuery.lut`
   by scoring directly from `codebook[idx] * prepared.rotated[dim]`.
4. Added explicit tests that the tiled `1536x4-bit` path keeps `qjl_signs`,
   `PreparedQuery.lut`, `PreparedQuery.sq`, and `qjl_scale` disabled, while
   a small `32x4-bit` path still keeps the QJL/LUT state live.

The pg rescan scaffold assertions in `src/lib.rs` were also corrected back to
their existing small-dimension contract. Those fixtures use `dim=4`, where
QJL is still active, so they should continue to expose non-empty LUT and QJL
state. The disabled-path change is intentionally limited to the tiled
production lane.

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

All three gates were rerun after the final code state and completed green.

## Result

Verified warm steady-state rerun:

```text
scripts/bench_sql_latency_verified_scratch.sh \
  --prefix tqhnsw_real_10k \
  --m 8 \
  --ef-search 40 \
  --cache-state warm-after-prime3 \
  --warmup-passes 3 \
  --session-mode per-cell
```

Measured result:

```text
m=8 ef_search=40 n=200 p50=11.024ms p95=13.244ms p99=15.491ms mean=11.111ms
```

Compared with the prior warm persistent baseline from packet `261`
(`p50=14.315ms`, `p99=17.613ms`, `mean=14.194ms`), this slice cut mean latency
by about `21.7%` and p50 by about `23.0%`.

This does not close C1, but it is a real warm-path improvement from removing
work that the no-QJL `1536x4-bit` scorer never consumed.
