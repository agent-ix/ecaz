# Review Request: AVX2 Vector Lane Accumulator

Commit: `ea41ff5`

Scope:
- `src/quant/prod.rs`

Summary:
- keep the aligned 3-bit decode from `10038`, but stop reducing AVX2 scorer work back into
  scalar on every 8-lane chunk
- accumulate MSE and QJL contributions in AVX2 registers across the loop body and do one lane
  reduction at the end of the vectorized region
- leave scalar behavior, dispatch policy, planner wiring, scan runtime, and FWHT unchanged

Before/after AVX2 scorer snapshot on this machine (same harness at `5000` iterations):
- before (`3adcc2d`, recorded in `review/10038`, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `1952.0 ns`, `score_ip_codes_lite/d1536_b4` `2145.6 ns`
- after (`ea41ff5`, rerun for this packet, auto `avx2+fma`):
  `score_ip_encoded/d1536_b4` `1597.0 ns`, `score_ip_codes_lite/d1536_b4` `2104.2 ns`

Current whole-harness stability snapshot on this machine (`20000` iterations):
- auto (`avx2+fma`): `fwht/2048` `4276.4 ns`, `fwht/4096` `8797.5 ns`,
  `score_ip_encoded/d1536_b4` `1596.0 ns`, `score_ip_codes_lite/d1536_b4` `2187.5 ns`

Experiment log:
- kept: AVX2 vector-lane accumulation in `score_ip_from_split_parts_avx2`; this removes repeated
  store-to-array plus scalar reduction from the hot loop without widening the dispatch surface
- observed from the matched `5000`-iteration runs: `score_ip_encoded` improved by about `18%`
  over `10038` on the same host/harness
- observed from the longer `20000`-iteration run: the `score_ip_encoded` win held at essentially
  the same absolute time, so this does not look like a one-off noisy sample
- kept: scalar behavior untouched; the targeted improvement is AVX2-only
- next candidate if we keep pushing B1 on this host: replace the remaining scalar LUT load loop in
  the AVX2 path, because that is now the obvious residual bottleneck

Please review:
- whether the vector accumulators remain obviously equivalent to the old per-iteration
  store-and-sum logic
- whether the new AVX2 reduction shape is narrow enough to justify as its own checkpoint
- whether the review packet has the right level of before/after detail for future B1 micro-slices
