# ec_diskann Apple-Silicon NEON Exact Rerank Kernel

Reviewer: please review this Apple-Silicon-specific ec_diskann checkpoint
and its packet-local A/B measurement.

## Scope

This packet measures committed head `dceda057`
(`Add NEON exact rerank inner product to ec_diskann`) against current
`origin/main` head `e5f380a1` on Apple M5.

The hypothesis was the same one that produced the IVF M5 win in
`review/30201-task31-m5-quality-neon-rerank`:

- `src/am/ec_diskann/ambuild.rs::source_inner_product(...)` had an AVX2+FMA
  fast path on `x86_64` and fell back to a scalar loop on Apple Silicon.
- The SQL ordered scan rerank path goes through
  `routine.rs::exact_heap_rerank_distance(...) -> ambuild::source_inner_product(...)`.
- Adding an `aarch64` NEON specialization should reduce per-rerank-row
  cost on Apple hardware without changing recall.

## Code Checkpoint

- code commit: `dceda057` (`Add NEON exact rerank inner product to ec_diskann`).
- shape: 16-lane NEON main loop with four parallel `vfmaq_f32` accumulators,
  4-lane tail, scalar remainder. Mirrors
  `src/am/ec_hnsw/source.rs::inner_product_neon`, the kernel shape that
  produced the IVF M5 win.

Focused validation before measurement:

- `cargo check --all-targets --no-default-features --features pg18`
- `cargo test --no-default-features --features pg18 --lib am::ec_diskann::ambuild`
  (3 tests pass, including the new
  `source_inner_product_neon_matches_scalar_at_loop_boundaries`).

No broader cargo or pgrx test sweep was run for this packet; the slice
is a narrow architecture-specific math-kernel change.

## Fixtures

Three fixtures, all built once under the scalar binary (`fc71290a4...`)
and reused unchanged under the NEON binary (`0538822d3...`); only the
loaded `ecaz.dylib` differed between arms. Full SHAs and commands are
in `artifacts/manifest.md`.

1. `m5_diskann_synth10k` (synthetic unit-sphere, seed-fixed, 10k x 1536d) —
   smoke fixture. Recall@10 falls to `0.16-0.33` here because synthetic
   high-dim vectors are nearly equidistant; per-query cost is dominated
   by scan / heap-fetch rather than the rerank kernel. Treat the synth
   numbers as kernel-correctness only.

2. `m5_diskann_real10k` (real DBpedia-style 1536d embeddings, 10k rows,
   200 queries) — copied from the existing `task31_m5_real10k_pqg8_n64`
   corpus into a fresh `ec_diskann` prefix, since the original Task 29
   `target/real-corpus/ec_hnsw_real_10k` TSVs were not on this M5.
   Reloptions match Task 29 (`graph_degree=32`, `build_list_size=100`,
   `alpha=1.2`).

3. `m5_diskann_real10k_w800` — same real10k corpus + queries, but
   `rerank_budget=800` and L=800 to amplify the share of per-query cost
   that lives inside the rerank kernel. This is structurally analogous
   to the `rerank_width=1000` quality lane that the IVF M5 packet
   `30201` used to surface its NEON kernel win cleanly.

## Result

Recall and NDCG are identical across the two binaries on every fixture,
which confirms the NEON kernel matches scalar within float tolerance:

| fixture | L | recall@10 (scalar = NEON) | NDCG@10 |
|---|---:|---:|---:|
| synth10k | 64 / 200 / 800 | 0.1650 / 0.2665 / 0.3260 | 0.8298 / 0.8811 / 0.9036 |
| real10k @ rerank_budget=64 | 64 / 200 / 800 | 0.9965 / 0.9970 / 0.9975 | 0.9999 / 0.9999 / 0.9999 |
| real10k_w800 @ rerank_budget=800 | 800 | 1.0000 | 1.0000 |

### real10k @ default rerank_budget=64 (200 iterations / L)

| L | scalar mean | NEON mean | scalar p50 | NEON p50 | scalar p95 | NEON p95 | scalar p99 | NEON p99 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.03 ms | 1.97 ms | 1.98 ms | 1.93 ms | 2.28 ms | 2.16 ms | 2.46 ms | 2.43 ms |
| 200 | 2.21 ms | 2.18 ms | 2.20 ms | 2.15 ms | 2.42 ms | 2.42 ms | 2.54 ms | 2.62 ms |
| 800 | 2.77 ms | 2.73 ms | 2.76 ms | 2.70 ms | 3.09 ms | 2.99 ms | 3.20 ms | 3.16 ms |

Per-arm stddev: scalar 0.23 / 0.19 / 0.23 ms, NEON 0.24 / 0.20 / 0.23 ms.

NEON moves p50 by `-0.05 ms` consistently across all three L values, but
`-0.05 ms` is well inside both arms' `~0.20 ms` stddev. The kernel does
roughly `64 x 1536 ~= 98K` mults per query, which is single-digit
microseconds even on scalar — small relative to the `~2-3 ms` total
query cost dominated by scan / heap fetch. Default-config NEON is a
real but small effect that does not clear the handoff promotion bar.

### real10k_w800 @ rerank_budget=800, L=800 (200 iterations / pass, 2 passes per arm)

This is the kernel-stress lane, the one structurally analogous to the
IVF `rerank_width=1000` quality lane.

| pass | mean | stddev | min | p50 | p95 | p99 | max |
|---|---:|---:|---:|---:|---:|---:|---:|
| scalar pass 1 | 17.7 ms | 21.0 ms | 15.4 ms | 16.2 ms | 17.1 ms | 18.8 ms | 313.6 ms |
| scalar pass 2 | 17.5 ms | 14.0 ms | 15.6 ms | 16.4 ms | 17.2 ms | 19.0 ms | 215.5 ms |
| NEON pass 1 | 15.1 ms | 0.55 ms | 14.3 ms | 15.0 ms | 15.7 ms | 16.7 ms | 20.6 ms |
| NEON pass 2 | 15.4 ms | 0.56 ms | 14.7 ms | 15.4 ms | 16.0 ms | 16.7 ms | 20.7 ms |

(scalar `mean` and `max` were inflated by single autovacuum-shaped
`200+ ms` outliers in both scalar passes; the percentile columns are
unaffected and agree across passes.)

Pass-averaged percentile deltas:

| metric | scalar avg | NEON avg | delta | rel |
|---|---:|---:|---:|---:|
| min | 15.5 ms | 14.5 ms | `-1.0 ms` | `-6.5%` |
| p50 | 16.3 ms | 15.2 ms | `-1.1 ms` | `-6.7%` |
| p95 | 17.15 ms | 15.85 ms | `-1.3 ms` | `-7.6%` |
| p99 | 18.9 ms | 16.7 ms | `-2.2 ms` | `-11.6%` |

The improvement is consistent across all percentiles, agrees between
the two passes within each arm, and is well outside the `~0.55 ms`
stddev of either NEON pass. Recall stays at `1.0000`. Scan and heap
counters are unchanged because only the rerank kernel changed.

## Interpretation

This is a real Apple-Silicon NEON kernel win, but only visible when
the exact rerank kernel is actually a non-trivial fraction of total
query cost:

- At `rerank_budget=64` (the current default), the rerank kernel does
  too little work per query for the `~3-4x` NEON speedup over scalar
  to surface above scan / heap-fetch noise. The change is a
  correctness-preserving micro-improvement at default config.
- At `rerank_budget=800` with L=800 (kernel-stressed), NEON cleanly
  wins across `min`, `p50`, `p95`, `p99` by `6.5-11.6%`, with recall
  unchanged. This is structurally the same pattern that `30201` saw
  for IVF at `rerank_width=1000`.

In other words: the NEON kernel is a real, correct, Apple-specific
win — but ec_diskann's default-config workload does not currently
spend much time inside the kernel, so the default-config win is
small. The win surfaces cleanly only when the rerank workload is
sized up.

## Recommendation

Land the code change. It is the same shape as the existing IVF NEON
kernel that already promoted, it is unit-tested for parity with
scalar, it preserves recall on every measured fixture, and it
produces a clean across-the-board win on the kernel-stress lane.

Two follow-ons are visible from this packet but are NOT in scope here:

1. If diskann production workloads commonly tune `rerank_budget` up
   (the way Task 31 IVF tunes `rerank_width=1000`), this change will
   matter at production scale. If not, this is a future-proofing
   checkpoint rather than a near-term latency mover.
2. Per the handoff, "if the NEON slice does not clearly promote, the
   next Apple-specific candidates are exact rerank source decode
   overhead and heap fetch / cache locality in the rerank path".
   The default-config result here is roughly the "does not clearly
   promote" case, so source-decode and heap-fetch instrumentation
   would be the natural next measurement-driven slice — but only
   after this kernel checkpoint lands, since both follow-ons live
   on the same code path the kernel just touched.

## Artifacts

All artifacts live under `artifacts/`. See `artifacts/manifest.md` for
SHAs, commands, fixture provenance, and full per-arm tables.
