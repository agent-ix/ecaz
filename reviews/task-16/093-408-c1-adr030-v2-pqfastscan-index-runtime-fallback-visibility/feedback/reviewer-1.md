## Feedback: PqFastScan Index Runtime Fallback Visibility

Read `resolve_pq_fastscan_traversal_score_mode_decision` in
`src/am/scan.rs:956–998`, the `resolve_grouped_traversal_score_mode`
call-through at :1000–1004, the new
`tqhnsw_debug_pq_fastscan_runtime_settings_for_index` in
`src/lib.rs:17076` and its non-pq_fastscan rejection at :17008–17023.

### What's right

- **Correctly responds to the exact concern I raised on packet
  `401`.** The earlier feedback flagged that an operator whose
  `pq_fastscan` index silently fell back from binary to grouped-PQ
  had no way to see the reason. This packet makes the *reason* a
  first-class enum (`PqFastScanTraversalScoreModeResolution`) with
  four named states — `EnvOverride`,
  `DefaultBinaryWithBinarySidecar`,
  `FallbackGroupedPqMissingBinarySidecar`,
  `NonPqFastScanStorage` — that a debug helper can render.
- **Existing scan path behavior preserved.**
  `resolve_grouped_traversal_score_mode` now call-throughs to
  `resolve_pq_fastscan_traversal_score_mode_decision(...).mode`. No
  behavior change at the hot path, just added observability. That
  is the right refactoring shape — add structure, keep the
  observable result identical.
- **Env override still wins and still labels itself as such.** An
  operator who set `TQVECTOR_PQ_FASTSCAN_TRAVERSAL_SCORE_MODE=pq`
  will see `EnvOverride` in the resolution field even if their
  index *also* has a binary sidecar. That is the right answer —
  the helper reports what the scan actually did, not what it
  would have done by default.
- **Non-pq_fastscan index properly rejected, not defaulted.**
  `tqhnsw_debug_pq_fastscan_runtime_settings_for_index` errors
  when pointed at a `turboquant` index rather than silently
  returning the `NonPqFastScanStorage` decision. That is the right
  choice for a debug helper named `_pq_fastscan_` — the
  resolution state exists internally for completeness but the
  helper's contract is "this is a pq_fastscan-specific tool."
- **Test coverage across all three interesting paths.** The pg
  tests prove normal binary default, metadata-edited fallback,
  and explicit env override — exactly the three states an
  operator investigating recall would want to distinguish.
- **Layout binary-word count exposed.** Surfacing
  `pq_fastscan_layout_binary_word_count` means an operator can
  tell at a glance whether the sidecar is absent (0) vs present
  and short (small non-zero) vs present and normal. That is the
  datum behind the fallback decision, so exposing it means the
  operator can reason about why their index doesn't have one.

### Concerns

1. **Index-aware helper is the third runtime-settings helper on
   the surface.** `tqhnsw_debug_pq_fastscan_runtime_settings()`,
   `tqhnsw_debug_adr030_runtime_settings()` (legacy compat), and
   now `tqhnsw_debug_pq_fastscan_runtime_settings_for_index(...)`.
   The naming distinction is logical but the proliferation is
   slightly noisy — three similarly-named helpers. Worth a
   follow-up packet to either consolidate or at least document
   which to reach for when.
2. **Fallback *mode* is surfaced; fallback *consequence* is not.**
   The helper reports that the scan fell back to grouped-PQ, but
   it does not report "which is likely to cost you X recall
   points vs binary." That is an advisory, not a required field,
   but for a debuggability helper it would be useful guidance —
   even a static sentence in the helper's function comment
   would help operators know whether the fallback is a real
   problem for their workload.
3. **Metadata-edited fixture is a synthetic path.** The fallback
   test proves behavior by editing metadata to strip the
   binary-sidecar flag. That is fine for unit coverage, but the
   real-world fallback path is "index built before the binary
   sidecar was persisted" — a legacy-shape index. A smoke test
   that actually builds an index with the sidecar bit off (if
   that path exists) or at least a comment explaining the
   fixture approximates that case would make the test's scope
   explicit.
4. **No index-aware rerank-mode resolution helper yet.** Packet
   `404` added source-aware default rerank, and that decision has
   the same "why did I get this mode" debuggability question as
   traversal score mode. The helper added here only resolves
   traversal; rerank mode is still reported as a global default.
   Natural follow-up: extend the index-aware helper to resolve
   rerank mode the same way.
5. **Same linker-gap caveat.** Three new pg tests added, all
   unexecuted. For this packet that risk is lower than most —
   the helper is pure observability with no side effects — but
   the tests are still only proven by `cargo check`.

### Observation

This is the right shape of follow-up to `401`. Instead of just
adding a field, the packet refactored the traversal-mode
resolution into a structured decision object, which is going to
pay off the next time this code needs to answer "why did this
scan behave this way." The enum-per-resolution-reason pattern is
also the pattern to reuse when packet `404`'s rerank-mode
decision needs the same treatment.
