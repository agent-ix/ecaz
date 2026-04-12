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

## Planned work

1. Make the no-QJL `4`-bit query-prep path skip LUT construction.
2. Avoid populating QJL-only quantizer state when QJL is disabled.
3. Add tests around the `1536`-dim, `4`-bit path so the disabled buffers stay
   visibly disabled.
4. Re-run the normal checkpoint gate and a verified warm per-cell benchmark.

## Exit criteria

- no-QJL `4`-bit prepared queries keep `lut` empty
- no-QJL quantizers do not populate QJL-only setup they will never use
- the change is validated through `cargo test`,
  `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`, and
  `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`
- the verified warm per-cell surface is rerun and recorded
