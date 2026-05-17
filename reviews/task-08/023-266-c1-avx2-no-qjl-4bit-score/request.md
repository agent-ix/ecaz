# Review Request: C1 AVX2 No-QJL 4-bit Score

## Context

Packet `265` removed unused query-prep state from the tiled `1536x4-bit`
no-QJL path and improved the verified warm steady-state `10K`, `m=8`,
`ef_search=40` cell from about `14.19ms mean` to `11.11ms mean`.

That is real progress, but it is still well above the warm-side `NFR-001`
target. Reviewer feedback on packets `257` and `264` both pointed at the same
remaining hotspot: the no-QJL `4-bit` scorer is still scalar on x86 even
though this production lane dominates current warm measurements.

## Problem

In `src/quant/prod.rs`:

- `score_ip_from_split_parts_no_qjl_4bit(...)` walks packed bytes one by one
  and scores two nibbles at a time with scalar loads and scalar multiplies
- the hot `1536x4-bit` production lane reaches that path whenever QJL is
  disabled
- the existing AVX2 scoring machinery only accelerates the QJL-enabled path

So the current warm path still leaves SIMD throughput on the table exactly
where the active production lane spends its scoring time.

## Experiment

I implemented and tested an x86_64 AVX2/FMA fast path for
`score_ip_from_split_parts_no_qjl_4bit(...)` in `src/quant/prod.rs`.

The implementation did pass the production-dimension scalar-vs-dispatched
agreement test after fixing the packed-nibble lane ordering bug
(`low0, high0, low1, high1, ...`), so this was a valid correctness probe.

## Result

The experiment did **not** justify landing the code.

Direct scorer microbench, using the existing `simd_bench` binary with forced
backends on the same local build:

```text
TQVECTOR_SIMD=scalar   score_ip_encoded/d1536_b4: ns_per_iter=14082.5
TQVECTOR_SIMD=avx2+fma score_ip_encoded/d1536_b4: ns_per_iter=14301.6
```

So the AVX2 path was slightly slower than the scalar path on the isolated
`1536x4-bit` scorer.

The verified warm SQL rerun was also too small to justify the extra code:

```text
packet 265 baseline: p50=11.024ms p95=13.244ms p99=15.491ms mean=11.111ms
packet 266 probe:    p50=11.102ms p95=13.137ms p99=14.654ms mean=11.010ms
```

That is essentially flat on mean/p50, with only a modest p99 improvement.

## Decision

I discarded the AVX2 implementation and reverted the code locally. This packet
remains as a recorded failed experiment because the result is still useful:
for the current no-QJL `1536x4-bit` lane, a simple AVX2 nibble-decode +
permute/blend scorer does **not** beat the existing scalar path.

## Next direction

The next C1 slice should move away from this particular scorer SIMD idea and
target a seam with clearer headroom than the current no-QJL `4-bit` scalar
loop.
