## Feedback: Scan Runtime Gate Removal

Read `validate_runtime_scan_format` wiring in `src/am/scan.rs`, the
removed env constant, and the runtime-settings probe reporting
`grouped_scan_enabled` as a fixed capability.

### What's right

- **Scan availability now follows persisted metadata, not process
  state.** This matches the reloption-driven build selection from
  packet 378 — build and scan now agree on "what format is this index"
  by reading the same metadata rather than two independent process
  env vars. That's the right invariant.
- **The exact-score error message is narrowed to the actual
  condition:** `tqhnsw grouped exact scoring requires the grouped
  cold rerank payload path`. Keeping that as a real capability gate
  instead of a blanket "grouped-v2 not supported" reject is the right
  call — the error now tells the caller *why* it can't proceed, not
  just that it can't.
- **Tests were converted from "rejection proves gate works" to
  "success proves path works."** Converting the old runtime-rejection
  case into a grouped ordered-scan smoke test is the right move once
  the gate is gone.

### Concerns

1. **`grouped_scan_enabled` as a fixed-`true` capability.** The
   runtime-settings probe still exports this column. If the intent is
   "capability is always on for any grouped-format index," a stable
   `true` is correct — but at that point the column is decorative and
   future readers may wonder if it means "is grouped scan
   implemented" (yes, fixed) or "is *this* index grouped" (depends on
   metadata). Worth a one-line docstring on the helper.

2. **Remaining grouped scan tuning env vars.** The packet lists
   window size, grouped score mode, rerank mode/source, and exact
   traversal as knobs that stay. Three of those feed into the
   user-visible error surface (e.g., rerank source column). Packet
   392 renames them, but the question of "which of these should still
   exist at merge, vs be hoisted to reloptions or GUC" is not
   addressed here. Task 15 doesn't strictly require doing that before
   merge, but it's worth a line in the ADR-032 followups list.

3. **Linker gap.** Same `cargo pgrx test pg17` boundary as the rest
   of the arc. The converted grouped ordered-scan smoke test is the
   new load-bearing proof that scan-via-metadata works end to end,
   and it was not executed locally.

### Observation

Good slice: narrow, removes an env var that was architecturally
stale, and the tests moved from negative to positive assertions
cleanly. The remaining runtime env surface is tuning/diagnostic debt,
not a correctness gap.
