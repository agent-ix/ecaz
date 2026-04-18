# Review Request: C1 Task16 TurboQuant Quantized Default

Current head at execution: `89398ec`

## Context

Packet `424` made turboquant participate in the shared live-rerank pipeline and
unblocked both deferred quantized rerank and source-backed heap-f32 rerank.

The immediate follow-up measurement on the isolated warm `50k, m=16, ef=128`
lane showed that those two rerank modes are not interchangeable for
turboquant:

- source-backed turboquant with default heap-f32 rerank: `5.220ms` mean SQL
  latency (`tmp/task16-turboquant-live-rerank-m16only.summary`)
- the same lane with explicit quantized rerank override: `3.005ms` mean SQL
  latency (`tmp/task16-turboquant-live-rerank-m16only-quantized.summary`)

So packet `424` correctly made heap-f32 *available* on turboquant, but leaving
it as the silent source-backed default made the serious operating point slower
than the quantized deferred-rerank path that actually closes the gap.

## What Landed

This packet narrows the default-policy layer without changing the shared rerank
implementation:

- source-backed `pq_fastscan` indexes still default to `heap_f32`
- source-less `pq_fastscan` indexes still default to `quantized`
- turboquant indexes now default to `quantized` regardless of whether
  `build_source_column` is present
- turboquant keeps `heap_f32` as an explicit runtime override, so the source
  fetch path remains available for experiments and targeted validation

## Code Shape

### `src/am/scan.rs`

- `default_grouped_rerank_mode(...)` now keys the implicit heap-f32 default on
  `StorageFormat::PqFastScan` instead of any index with `build_source_column`
- `default_grouped_rerank_mode_resolution(...)` now reports the turboquant case
  explicitly as `default_quantized_turboquant_storage`
- unit coverage now splits the policy by storage format:
  - source-backed `pq_fastscan` => `heap_f32`
  - source-backed `turboquant` => `quantized`
  - source-less `pq_fastscan` => `quantized`

### `src/lib.rs`

- the turboquant heap-rerank profile test now opts into `heap_f32` explicitly
  via `TQVECTOR_PQ_FASTSCAN_RERANK_MODE=heap_f32`
- added source-backed turboquant coverage proving the default path stays on
  quantized rerank and leaves heap counters inert

## Validation

Green on this head:

- `cargo test`
- `bash scripts/run_pgrx_pg17_test.sh`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Focused runtime coverage now proves both turboquant paths:

- default source-backed turboquant rerank stays quantized
- explicit source-backed turboquant heap-f32 override still works

## Readout

This packet is intentionally policy-only. It does not remove turboquant heap
rerank, and it does not change the deferred-rerank machinery from `424`. It
just makes the default line up with the measurement we already had:
turboquant’s serious operating point wants deferred quantized rerank by
default, not silent source-backed heap fetches.
