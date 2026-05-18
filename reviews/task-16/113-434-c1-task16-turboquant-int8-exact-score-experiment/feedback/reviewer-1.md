## Feedback: Int8 exact-score scan-path seam — ACCEPTED

Verified against:

- commit `e0ba7ee` adding
  `TQVECTOR_TURBOQUANT_EXACT_SCORE_MODE` env-gated scan path
- `src/am/scan.rs` prepared scan state now holds an optional int8
  query alongside exact / binary-sign
- `score_scan_element_result(...)` dispatch gated on env
- `pg_test_turboquant_scan_stage_profile_int8_mode` and
  `pg_test_turboquant_exact_score_mode_rejects_invalid_env`
- debug stage profile reports effective score mode through existing
  columns

### What's right

- **Default-off and opt-in.** Current-head behavior is unchanged
  unless the env is set. This is the correct shape for an
  experimental scorer seam: measurable without risk of silently
  changing production behavior.
- **Lane-gated.** Only the no-QJL `1536@4` TurboQuant lane may use
  the override; unsupported lanes error immediately rather than
  silently running the wrong scorer. Good guardrail.
- **Cache-preserving wiring.** Choosing to go through the scan-local
  score cache (vs. a one-off uncached comparison path) was a
  deliberate call in §3 and is the right one. Bypassing the cache
  would have underestimated the real runtime win by paying re-score
  cost on every frontier re-visit.
- **Debug surface reuses existing columns.** The stage-profile
  `turboquant_exact_score_mode` / `_uses_lut` / `_uses_qjl` fields
  were already there from packet `423`; not inventing a second
  TurboQuant debug helper first is the right instinct.
- **Env validation pinned.** The invalid-env test locks the accepted
  values list. Packet `436` later extends it to four values; the
  test pattern carries forward cleanly.

### Concerns

1. **Env name is TurboQuant-specific, but ADR-043's `tqvec` type
   (PROPOSED) could reopen the "which column feeds which scorer"
   question.** Not a blocker for this experimental seam — just
   worth noting the env is deliberately experimental (not a
   persisted reloption) so renaming or replacing it later is cheap.
2. **No score-parity assertion between `int8_approx` and `exact`
   at the scan level.** Packet `433`'s offline study proved the
   quantizer helpers are near-exact, but the live scan path also
   applies binary prefilter, deferred rerank, and cache hit/miss
   logic. A single integration test that asserts the two modes
   produce "close enough" top-K on a small fixture would catch a
   regression in any of those seams. Current coverage only asserts
   the stage profile reports the right mode label.
3. **Test name `pg_test_turboquant_scan_stage_profile_int8_mode`
   verifies the mode label and the presence of rerank/prefilter
   counters.** It does not assert a latency or result-count
   invariant. That's fine for an experimental seam, but it means
   the test won't catch a scorer that silently returns zero
   results — exactly the shape of the V3 concurrency bug
   surfaced in packet `437`. Not blaming this packet; just a
   missing invariant worth adding.

### Call

Accepted. Clean opt-in seam, right default-off posture, and the
right use of the existing debug surface. Paired with packet `436`
to extend to lever-4 modes and with packet `437` for the runtime
verdict.
