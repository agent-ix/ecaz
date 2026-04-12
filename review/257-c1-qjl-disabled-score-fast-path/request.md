# Review Request: C1 QJL-Disabled Score Fast Path

## Context

Packet `256` materially improved the graph side of the C1 scan path. The
current best verified canonical real-`10k` `m=8` surface on `main` is now:

- `ef_search=40`: mean `69.855ms`
- `ef_search=200`: mean `124.238ms`

The remaining hot-path buckets for the representative `id=10000` probe are now
dominated by layer-0 seed work and candidate scoring:

- `ef_search=40`
  - layer-0 seed elapsed: `15.802ms`
  - candidate score elapsed: `16.217ms`
- `ef_search=200`
  - layer-0 seed elapsed: `87.147ms`
  - candidate score elapsed: `76.667ms`

On the real-corpus lane, scoring runs through the 1536-dim, 4-bit,
QJL-disabled path. `qjl_enabled(1536, 4)` is false because the tiled 1536-dim
FWHT compatibility path is active, so `score_ip_from_parts` currently falls
back to the scalar non-QJL scoring loop.

## Problem

The graph-side runtime is no longer the clean first target. The scan still
spends a large fraction of its time in quantized candidate scoring, and the
hot real-corpus path is on the exact 4-bit production configuration where the
codebook is tiny and highly regular.

That makes the scalar non-QJL score loop a likely high-leverage C1 seam.

## Planned work

1. Confirm the current non-QJL 4-bit scoring path and establish a local
   microbench baseline on the existing SIMD bench harness.
2. Add a narrow fast path for the QJL-disabled 4-bit score loop, keeping the
   existing scalar implementation as the reference path.
3. Validate correctness against the existing dispatched-vs-scalar score tests.
4. Re-run the required checkpoint gate and then measure whether the scan hot
   path and canonical real-corpus surface move materially.

## Exit criteria

- the active packet records the exact score-path change
- validation is green (`cargo test`, `cargo pgrx test pg17`, clippy)
- the packet captures both microbench or hot-path evidence and the scan-level
  effect on C1
- if the fast path does not buy real scan latency, the packet says so plainly
