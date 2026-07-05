# Task 28 IVF Score-Bound Prune Trial

## Scope

This packet records a negative A7 implementation trial. No code from this trial landed.

Baseline branch head after backing out the trial: `591c10adf5eb2b859d5da77274d9bc3a9ea074bd`.

## Trial Summary

Two true-bound variants were tested against the current TurboQuant no-QJL 4-bit LUT scan path:

- Per-dimension suffix upper-bound check against the current pre-rerank frontier.
- Coarser/byte-LUT variants intended to reduce branch overhead and use a byte-level suffix bound.

Both were safe in unit tests, but both were negative on the 10k DBPedia n64/w25 latency surface. The byte-LUT scorer improved the isolated release kernel (`1331 ns` in packet 30076 to `1083 ns` here), but the larger per-query prepared state hurt the dev PG scan benchmark.

## Results

Packet 30073 baseline for this surface:

- `nprobe=32`: p50 `60.5 ms`, p95 `75.6 ms`
- `nprobe=48`: p50 `84.5 ms`, p95 `99.8 ms`

Negative trial results:

- Per-dimension bound check:
  - `nprobe=32`: p50 `71.6 ms`, p95 `82.5 ms`
  - `nprobe=48`: p50 `95.0 ms`, p95 `102.7 ms`
- Coarse bound check:
  - `nprobe=32`: p50 `110.1 ms`, p95 `118.3 ms`
  - `nprobe=48`: p50 `159.2 ms`, p95 `169.5 ms`
- Byte-LUT scorer plus byte-level bound:
  - `nprobe=32`: p50 `101.1 ms`, p95 `107.7 ms`
  - `nprobe=48`: p50 `142.0 ms`, p95 `150.3 ms`

## Decision

Do not land this A7 approach. A7 remains open.

The next A7 attempt should avoid adding substantial per-query prepared state in the dev PG path. The useful clue from this packet is that a denser scorer can win in isolation, but the scan path needs either a cheaper prepared representation or a posting-layout change that amortizes byte-level scoring without a large query-prep penalty.

No DiskANN work is included in this packet.

## Artifacts

- `artifacts/latency_10k_n64w25_nprobe32_48.log`
- `artifacts/latency_10k_n64w25_nprobe32_48_coarse.log`
- `artifacts/latency_10k_n64w25_nprobe32_48_byte_lut.log`
- `artifacts/simd_bench_byte_lut.log`
- `artifacts/manifest.md`
