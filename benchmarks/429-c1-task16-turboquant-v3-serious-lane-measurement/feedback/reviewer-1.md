## Feedback: V3 serious-lane measurement — ACCEPTED

Verified against:

- commit `735565a` adding artifacts and request
- cited log/CSV artifacts (`tmp/task16-turboquant-v3-heapf32-
  m16only.summary`, `-stageprofile.csv`, `-quantized-` pair)
- matching real-corpus recall summary TSVs

### What's right

- **Isolates "V3 layout" from "rerank mode" on one index.** Both
  latency cells ran against the same rebuilt V3 index, with only
  `--rerank-mode` differing. That isolates the layout effect from
  the policy effect cleanly. Without this, the heap-f32 regression
  could have been misattributed.
- **Scratch rebuild is documented.** `/tmp/tqvector_pgrx_home`
  didn't exist and was recreated; the packet names every step. A
  future reviewer can replay without reverse-engineering.
- **Conclusion is corrected by the data.** Going into the packet
  the expectation was "V3 helps the recall-preserving lane." The
  data said otherwise (`5.220 → 6.086ms` on heap-f32), and the
  packet published that instead of massaging it.
- **Stage profile explains the regression.** Rerank bucket is
  `1.22ms` on heap-f32 vs `45us` on quantized — the bottleneck is
  not traversal/exact-score, it's the heap fetch/decode path. That
  is a non-obvious finding and it licenses packet `430`'s probe.
- **Quantized win is reported honestly.** `2.958 → 2.158ms`
  (`-27%`) is a real improvement, but the packet correctly refuses
  to treat it as closing the task. The recall target is still the
  decision criterion.

### Concerns

1. **Three latency cells total, n=1 each.** Fine for a directional
   readout; not enough to attach a confidence interval to the
   `+16.59%` heap-f32 regression. The subsequent `431`/`432`
   measurement noise (spread `4.65 – 6.36ms` on three runs)
   retroactively suggests this cell was also probably mid-spread.
   Worth carrying 2–3 reruns per cell on future serious-lane runs.
2. **`ef_search = 128` is held constant, which is the right
   comparison cell but not the only interesting one.** Task 16's
   decision really depends on the recall-preserving curve shape
   across `ef_search`, not a single cell. Not a blocker for this
   packet's specific claim; worth calling out as explicit follow-on
   measurement before closing task 16.
3. **Exact-score calls per query dropped from `~1605` to `1.00`
   on heap-f32.** That is a near-complete elimination and is
   reported correctly. But this also means the "V3 heap-f32 stage
   profile" for `exact_score_calls_mean = 1.00` is effectively
   measuring one query's worst case, not a distribution. Worth a
   note that the exact-score bucket is no longer a meaningful
   comparator on this lane.

### Call

Accepted. The packet identifies the real remaining bottleneck (heap
rerank source fetch/decode) instead of claiming V3 closed the gap.
That insight directly motivated packet `430` and was the right
readout.
