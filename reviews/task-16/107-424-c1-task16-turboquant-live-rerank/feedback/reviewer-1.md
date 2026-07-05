## Feedback: TurboQuant live rerank — ACCEPTED

Verified against:

- commit `a94d98d` (`Defer turboquant exact scoring into live rerank`)
- `src/am/scan.rs` additions:
  - `turboquant_binary_live_rerank_enabled(...)` predicate
  - the `configure_grouped_heap_rerank_state(...)` fix removing the
    `PqFastScan`-only guard
- new stage-profile counters
  (`binary_prefilter_score_calls`/`_elapsed_us`/`survivor_candidates`)

### What's right

- **Packet and code match.** The scan change actually does what the
  request says: binary-prefilter traversal runs first, the exact
  comparison is deferred into the shared rerank window, and the
  selector honors quantized vs. heap-f32 based on the existing mode
  decision.
- **Removes a real dead-end.** Before this packet,
  `configure_grouped_heap_rerank_state` threw away a `HeapF32`
  decision for every non-`pq_fastscan` storage and force-downgraded
  to quantized. That was a silent correctness hazard, not just a
  missing perf feature, because it made heap-f32-resolved turboquant
  scans behave differently from the decision logic above them. Glad
  this is gone.
- **Stage profile stays in sync with runtime.** Updating
  `debug_turboquant_scan_stage_profile` to subtract rerank elapsed
  from traversal residual (not just prefilter/exact) keeps the
  residual meaningful after this packet. Would have been easy to
  skip.
- **Scope discipline.** No new lever measurement claims attached.
  Correctly punts that to a later packet.

### Concerns

1. **No default-policy adjustment here.** This packet makes heap-f32
   *available* on source-backed turboquant but keeps it as the
   silent default. Packet `425` later shows that default is the
   wrong one on the serious lane. Would have been cleaner to land
   the two together since they share `configure_grouped_heap_rerank_
   state`, but splitting into two reviewable diffs is defensible.
2. **`turboquant_binary_live_rerank_enabled(...)` keys off
   `binary_sign_query(...).is_some()`.** That's right for today's
   `1536x4` lane, but if future code widens binary-sign availability
   to lanes where exact-score per survivor is still cheap, this
   predicate will opt *everyone* into deferred rerank. Worth a
   comment naming the assumption, or at least a test pinning the
   enable condition.
3. **Focused coverage names counts, not scores.** The new rerank
   quantized/heap profile tests verify the buckets report non-zero
   rerank work and fewer traversal exact-score calls than
   prefilter-survivors. Neither of those pins the *correctness* of
   the deferred result vs. the immediate-exact-score baseline. Not a
   blocker (the measurement packets that follow catch recall drift),
   but worth an explicit score-parity assertion between deferred and
   undeferred turboquant rerank on a small fixture.

### Call

Accepted. Code matches the narrative, and the fix to
`configure_grouped_heap_rerank_state(...)` is a quiet but real
correctness win on its own.
