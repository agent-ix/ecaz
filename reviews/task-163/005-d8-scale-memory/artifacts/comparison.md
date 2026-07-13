# D8 scale and quality comparison

## Current exact-SHA memory / NOTICE matrix

All rows use `build_shards=4`, `closure_epsilon=0.1`, RaBitQ, seed 42, and a
25 ms `/proc/<backend>/status` sample interval around `CREATE INDEX`.

| Scale | Backend HWM before / peak KiB | Build HWM delta KiB | Spill bytes | Completion high-water bytes | Stitch retained bytes | Stitch / spill |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 10k | 169,976 / 397,428 | 227,452 | 1,283,964 | 464,244 | 35,784 | 2.787% |
| 50k | 170,252 / 1,185,676 | 1,015,424 | 8,505,972 | 3,307,900 | 36,104 | 0.424% |
| 100k | 170,300 / 2,170,028 | 1,999,728 | 17,524,784 | 6,289,260 | 36,240 | 0.207% |

The whole build's backend HWM grows because source vectors, the in-progress
Vamana graphs, and the required output graph are not D8 stitch scratch. The
specific stitch residency named by FR-077-CON-4 stays essentially flat:
35,784 to 36,240 bytes (+456 bytes) while the encoded spill grows 13.65x.

`build_peak_completion_bytes` answers packet 004 P3-1 directly. It includes
flat encoded completions queued, blocked in worker sends, or currently being
drained to `BufFile`. Its observed high-water is 36.16%, 38.89%, and 35.89% of
the full spill. It is intentionally a scheduling-dependent diagnostic and is
excluded only from the graph-determinism equality assertion; every graph and
deterministic stitch statistic remains identical across repeated builds.

## Pre-D8 versus current 10k quality

Baseline is immutable packet 001's `m1_10k_stitch` arm at source
`a375d56dd70f364f8c2389201e5524e578f0ff14`, before the BufFile/cursor D8
implementation. Candidate is packet 005 at `cec8abba1770dc500a890c7ad57a932deae4c51c`.
Both use the same 10k corpus SHA, queries, 200 queries / 2,000 recall@10
trials, seed, reloptions, and search-width sweep.

| Search width | Pre-D8 recall@10 | Current recall@10 | Delta |
| ---: | ---: | ---: | ---: |
| 16 | 0.9950 | 0.9950 | 0.0000 |
| 32 | 0.9985 | 0.9985 | 0.0000 |
| 64 | 1.0000 | 1.0000 | 0.0000 |
| 100 | 1.0000 | 1.0000 | 0.0000 |
| 200 | 1.0000 | 1.0000 | 0.0000 |

All five exact-equality suite thresholds pass. The matching percentile and
CI rows in the two recall artifacts also agree, so the streamed spill/cursor
implementation has no observed 10k quality effect.
