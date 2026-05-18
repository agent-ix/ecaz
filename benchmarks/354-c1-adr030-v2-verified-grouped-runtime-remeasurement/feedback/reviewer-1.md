## Feedback: ADR-030 v2 Verified Grouped Runtime Remeasurement

Read the whole request end-to-end. This is a measurement-only packet
with no production code, so the review is about the finding, the
discipline of the discovery, and what the numbers actually say.

### What's right

- **This is the right catch at the right time.** Using
  `tests.tqhnsw_debug_grouped_scan_windowed_summary(...)` to check
  whether the supposed grouped indexes were actually grouped-v2 on
  disk is exactly the structural verification that was missing from
  packet 352. `grouped_result_count = 0` is a decisive, cheap
  signal.
- **Invalidation is honest.** The outcome section leads with
  "packet 352's scratch grouped numbers were not measuring real
  grouped-v2 runtime" — not "there's some concern about earlier
  numbers." That's the framing this needs. Anyone reading the
  branch history now sees 352's conclusions as superseded, which
  is correct.
- **Format verification after rebuild, documented.** The lines
  71-78 record `emitted_result_count = 40`, `grouped_result_count =
  40`, `compared_result_count = 40` after rebuild — the same
  diagnostic used for invalidation now used for confirmation.
  Symmetric. Good.
- **Root cause is structural, not one-off.** The mechanism was
  "scratch cluster wasn't restarted with the ADR-030 build gate
  before building the grouped indexes." That's the kind of
  operational slip that will happen again if not addressed in the
  packet template. The "Next Slice" point #3 ("re-run any future
  corpus measurements only after verifying the scratch index
  format first") captures this as a process change, not just a
  one-time fix.
- **Keeps both windows comparable on the same re-run.** 50k at
  both `window=8` and `window=16`; 10k at `window=16`. That's the
  minimum to make the headline claim ("wider window materially
  helps, but doesn't close the gap") defensible.
- **Emitted-set window summary is the most informative table in
  the packet.** Lines 90-95 showing window=1/4/8/16 Spearman
  going 0.53 / 0.73 / 0.87 / 0.97 tells you where the saturation
  point is *within the emitted set*. That's what you need to
  decide if widening further is worth more runtime.

### What the verified data actually says

The headline finding is much worse than 352 claimed. Verified
grouped-v2:

**50k grouped at window=8 vs scalar:**
- recall@10: 0.58-0.61 (grouped) vs 0.86-0.89 (scalar) — a
  **25-30 point gap**, not the 1-2 point gap 352 reported
- latency: grouped is 1.5-2x faster (0.98-2.81ms vs 1.40-4.52ms)

**50k grouped at window=16 vs scalar:**
- recall@10: 0.66-0.68 (grouped) vs 0.86-0.89 (scalar) —
  **20-23 point gap**
- latency: grouped still 1.5-2x faster

**10k grouped at window=16 vs scalar:**
- recall@10: 0.80-0.82 (grouped) vs 0.93-0.94 (scalar) —
  **~13 point gap**
- latency: grouped 3-5x faster

So the actual situation is: the grouped lane is *much* faster than
scalar, and substantially less accurate — to a degree that makes
it not competitive as a drop-in replacement. The latency win is
real but the recall gap is a first-order problem.

### Concerns

1. **The emitted-set window diagnostics say the problem is at
   least partially structural, not just rerank-width.** Line 94
   shows "exact-top4 fully inside rerank window = 0.5400" at
   window=16. That means even looking at 16 candidates, only 54%
   of queries have all four exact-top4 candidates *present*. The
   other 46% have at least one of the top-4 truly best candidates
   missing from the emitted set entirely. No rerank prefix can
   recover a candidate that wasn't emitted.

   That's a candidate-set quality problem upstream of rerank —
   either:
   - HNSW traversal under grouped scoring is drifting away from
     the true neighborhood (approximate grouped scores mislead
     the beam), or
   - the graph built under the ADR-030 build gate is
     structurally worse than the scalar build (fewer or worse
     edges at the top levels).

   Both hypotheses are testable. (a) would show up as "grouped
   scan with the *same* graph as scalar still underperforms."
   (b) would show up as "grouped scan forced to use a scalar-built
   graph recovers most of the recall gap." Worth teeing up this
   experiment before assuming wider rerank is the path forward.

2. **The 10k verified numbers are suspicious in a different way:
   the exact-quantized ceiling dropped from 0.9310 (352's
   apparently-scalar-masquerading-as-grouped) to 0.7965 (verified
   grouped).** That's a 0.13-point drop in the *ceiling* between
   the two builds. If the exact-quantized metric is computed
   against the index's own quantized codes, then the grouped
   4-bit nibble codes are significantly lossier than the scalar
   8-bit codes on this corpus. The quantizer itself is leaving
   recall on the table before HNSW traversal even gets to choose.

   For 50k the ceiling is 0.8560 in 352 and 0.8600 in 354 —
   much closer. So the ceiling drop is a 10k phenomenon. That's
   odd and worth understanding: smaller corpora have fewer
   candidates to disambiguate, so quantization errors should
   matter *less*, not more. Unless the grouped quantizer is
   trained on a tiny sample and underperforms on short data.
   Worth a poke at the training-sample size / centroid count
   ratio for 10k.

3. **Same-cluster scalar numbers changed between 352 and 354.**
   Scalar 50k @ ef=200 in 352: 0.8940 recall / 4.4875ms. Scalar
   50k @ ef=200 in 354: 0.8940 recall / 4.5234ms. Those match
   (good). But scalar 10k @ ef=200 in 352: 0.9400 / 8.1569ms vs
   in 354: 0.9400 / 8.8769ms — latency drifted ~0.7ms (~9%)
   between packets on the same scalar index with no reindex. That
   tells you the scratch cluster has enough noise that single-pass
   latency numbers can wobble by ~10%. Worth warmup passes plus
   multi-pass averaging before treating latency deltas under 10%
   as signal.

4. **Scratch env soup is getting complex.** At this point the
   postmaster needs:
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD=1`
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN=1`
   - `TQVECTOR_EXPERIMENTAL_ADR030_V2_SCAN_WINDOW=8` (or 16)

   Silently misconfiguring any of these produces either the 352
   bug (build gate off → scalar-on-disk under a grouped name) or a
   `window=4` default when you meant `16`. The verification step
   in this packet (use the debug SQL summary to confirm grouped on
   disk) only catches one of these. Before the next measurement
   packet, consider adding a "settings audit" step at the top of
   each run: a one-shot query that reports which ADR-030 envs the
   current backend actually sees. That would have caught 352 in
   ~30 seconds and it will catch the next variant automatically.
   (This is a `current_setting('my.var', true)` pattern if anything
   gets promoted to a GUC, which is another argument for
   promoting the window to a GUC sooner rather than later.)

5. **The ceiling column is still a reading hazard.** Same caveat I
   flagged on 352: "exact-quantized Recall@10" is a lossy proxy,
   not an attainable ceiling. In 354 at 50k grouped, the grouped
   Recall@10 at ef=200 (0.6820) is *below* the exact-quantized
   ceiling (0.8600). That's expected — grouped is failing to
   return what even its own quantized ground-truth says it should.
   So the ceiling column is at least *useful* here as a "where
   grouped could potentially reach with a perfect traversal over
   its own quantization." A 20-point gap to that ceiling is an
   enormous lever — more than widening the rerank window could
   plausibly recover, and consistent with concern #1 above.

### Observation

This packet is the turning point for the ADR-030 investigation. The
whole branch from packet ~330 onward was building toward the
narrative "grouped-v2 has small recall gap but big latency win on
real corpus — ship it behind a gate." Verification says that
narrative was false. The real narrative is "grouped-v2 has a big
recall gap and a big latency win" — a more interesting tradeoff,
but not a drop-in replacement.

That changes the framing of the next slice. The question is no
longer "what's the right rerank width for a grouped-vs-scalar
operating point?" — it's "is grouped-v2 recoverable as a
general-purpose operating point, or does it become a specialized
path (e.g., for k=1 latency-sensitive queries, or for systems that
can afford deeper rerank)?"

The "Next Slice" list is right to stay on the verified grouped lane
and target the quality gap directly. I'd add one experiment to the
list: measure the same 50k grouped recall on a *scalar-built*
graph — that is, build HNSW under scalar rules but score candidates
using grouped-v2 scoring at scan time. If that recovers most of the
recall, the problem is the grouped-built graph structure, not the
grouped scoring itself. If it doesn't, the problem is the 4-bit
quantization fidelity for this candidate-selection role.

### Measurement gap status

- corpus-scale recall at 50k: **closed with negative result** —
  grouped trails scalar by 20-23 points at window=16. Not ready.
- corpus-scale recall at 10k: **closed with negative result** —
  grouped trails scalar by ~13 points at window=16. Not ready.
- latency: **genuinely positive** — grouped is 1.5-5x faster
  consistently across corpora and ef. That win is robust.
- emitted-set quality: **diagnosed** — window=16 not yet
  saturated; ~46% of queries still miss at least one top-4 truly
  best candidate, which is a candidate-set problem, not a rerank
  problem.
- verification hygiene: **diagnosed and partially addressed** —
  the debug-summary check is now the required first step, but the
  scratch env surface could use a settings-audit helper before
  the next measurement packet.

This is the most important single packet in the recent sequence
because it invalidates a wrong conclusion without flinching.
Future measurement packets should cite this as the baseline rather
than 352.
