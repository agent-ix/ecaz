# Task 206 versus Task 215 latency reconciliation

The accepted Task 206 evidence and the Task 215 release matrix are not
absolute-latency comparisons of the same measured workload. They share the
wide-beam regime (`BW64/H8`, persisted-head seed count 128), three sharded
owners, `build_shards=1`, graph degree 32, head cap 4096, 200 queries, and
warm-cache single-concurrency latency sampling. The per-scale query SHA-256
values also match:

| scale | query SHA-256 |
| --- | --- |
| 10k | `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` |
| 50k | `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` |
| 100k | `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` |

The material work surface differs materially:

| axis | Task 206 accepted matrix | Task 215 release matrix |
| --- | --- | --- |
| query output | `top_k=200` | `top_k=10` |
| candidate heap | `L=200` | requested `L=32`, runtime-effective `L=64` under BW64 |
| extension | Task 206 source/build (`a6289dddf` decision run; feature/diagnostic variants also exist) | normal release source `ea51a9c8b...`, attribution feature absent |
| scan notice | enabled in the Task 206 diagnostic configuration | absent from the release decision arm |
| staged-dir naming | historical `task106_*` / `task111a*` staged surfaces | `data/staged-current` |
| cache/load | warm, single-concurrency protocol; no concurrent-load claim in either packet | warm, single-concurrency protocol; no concurrent-load claim |

The top-k/L difference changes the materialization path itself: Task 206 asks
the owners and executor to retain and materialize up to 200 results with a
200-entry heap, while Task 215 asks for 10 results with an effective 64-entry
heap. This is the primary recorded explanation for why Task 206's accepted
absolute means (roughly 194–231 ms) are about 8x above Task 215's 22.6–31.6
ms; the existing artifacts do not isolate the top-k effect from the source
build/feature difference, so no stronger causal percentage is claimed.

The release-generalizable decision boundary is therefore explicit:

- Task 215's normal-release A/B is authoritative for the shipped-default
  decision at the tested production workload (`top_k=10`, effective `L=64`
  for the candidate). It says STOP and retain BW4/H100.
- Task 206 remains valid evidence that BW64/H8 was a promising regime under
  its high-output diagnostic workload, but its absolute latency must not be
  reused as a normal-release BW64/H8 forecast. A top-k-200 production claim
  requires a separate normal-release A/B at top-k 200.

This note is the cross-packet annotation for future re-use of the Task 206
recommendation; no Task 206 or Task 215 measurement is being re-run.
