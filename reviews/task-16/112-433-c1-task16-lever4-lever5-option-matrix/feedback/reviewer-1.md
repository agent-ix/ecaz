## Feedback: Lever-4/Lever-5 offline option matrix — ACCEPTED as offline data, DO NOT close levers 4/5 on this packet alone

Verified against:

- commit `7a0df32` adding `PreparedLutNoQjl4BitQuery`,
  `PreparedTiledLutNoQjl4BitQuery`, and matching
  `prepare_ip_query_*` / `score_ip_from_parts_*` helpers
- extended `approx_score_study` with `full-lut` / `tiled-lut`
  modes and `--tile-size`
- exactness tests proving full-LUT and tiled-LUT match the direct
  scorer bit-for-bit on the no-QJL `1536@4` lane
- four study artifacts in `tmp/`

### What's right

- **Measurement surface is built, not argued.** The packet adds the
  full-LUT and tiled-LUT study modes, proves them exact against the
  direct scorer on the serious lane, and runs the matrix. That is
  exactly the method the user asked for — data, not inference.
- **Exactness locked in.** `rho = 1.0000`, `top10_overlap = 1.0000`
  for both full and tiled LUT, with quantizer tests that fail if a
  future refactor drifts the LUT path. The comparison is not
  contaminated by accidental approximation.
- **Int8 approx characterized on ranking, not just speed.**
  `pearson_r = 1.0000`, `top10_overlap = 0.9980`, `top10 captured by
  top20 = 1.0000`. Those together say the `+0.2%` overlap loss is
  recoverable with a modest over-fetch — a useable property, not
  just a raw speed number.
- **Binary sign reported both cached and derived.** `25.7ns` vs
  `5598ns` is a ~200× gap and the packet names it. A reader cannot
  conclude "binary sign = `25ns`" without seeing that the prep cost
  matters when it is not cached.

### Concerns

1. **Offline microbench is not the live scan path.** The readout
   in §1 says "lever 4 is not justified on the current serious
   lane" based on a single-query-vs-50k scorer loop. The live
   scan-path measurement in packet `437` later shows `full_lut`
   and `tiled_lut` *are* materially faster on the quantized
   runtime (`-16%`). That divergence is the important finding
   here: offline microbench under-predicted lever 4.

   **Don't treat this packet's §1 as closing lever 4.** It closed
   the offline measurement step — not the decision. That was the
   correct scope; please don't let the §1 wording bleed into later
   packets as if it were a runtime verdict.
2. **`bench_iters = 4`, `query_count = 50`.** Small sample for a
   nanosecond-level microbench. No confidence interval reported.
   The `~1300 ns/score` exact baseline across three of four study
   runs suggests the variance is low, but worth saying so
   explicitly or adding a spread column.
3. **Binary derived-prep cost at `5598ns` is surprising.** That is
   4×+ the exact scorer cost per score. Worth a one-liner on
   whether the derived-prep path is actually used anywhere in the
   live scan, or whether it's purely a study artifact. If it's in
   any live path that bypasses the cache, this is a latent hazard.
4. **`full-lut` vs `tiled-lut` difference is almost noise (`1304`
   vs `1480ns`).** The readout calls tiled "strictly worse" —
   which offline, at these sample sizes, is pushing the data. It
   is not the same claim as "tiled has no plausible runtime win";
   the runtime matrix in packet `437` confirms tiled is neck-and-
   neck with full on live. Worth the softer framing.

### Call

Accepted as an offline scorer study packet. It built the
measurement surface that packet `437` then used for the real
verdict. The §1 "lever 4 is not justified" framing is stronger than
the evidence supports and should not be treated as a runtime
decision — the runtime matrix in packet `437` is the decision cell,
and it tells a different story for lever 4.
