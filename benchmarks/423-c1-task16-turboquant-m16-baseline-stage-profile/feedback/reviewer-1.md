## Feedback: TurboQuant m16 baseline stage profile — ACCEPTED

Verified against:

- commit `a7c1da1` adding
  `tests.tqhnsw_debug_turboquant_scan_stage_profile`
- `src/am/scan.rs` binary-prefilter plumbing already present on
  turboquant paths (not PqFastScan-only)
- `src/quant/prod.rs` + `src/quant/rotation.rs` confirming the
  `1536`-dim `4`-bit lane is already no-QJL

### What's right

- **Didn't blindly replay the task text.** The most valuable thing
  in this packet is §`Current-code reality`: both of lever-1 and the
  QJL-accumulate stage were already present/absent on current head,
  not something still to port. A baseline packet that caught this
  before any lever code landed is exactly the right first move.
- **Stage-profile helper is scoped correctly.** The new debug
  wrapper is a test-only surface reporting the current turboquant
  counters, not a new public API. It gives lever-2 measurement
  something code-backed to compare against instead of inferring from
  generic scan counters.
- **Label mapping table is explicit.** The "requested stage →
  current-head interpretation" table makes it easy for a follow-on
  packet to use the same labels without re-deriving what counts as
  "inactive" on this lane.
- **A/B sanity check on the binary prefilter.** The toggle through
  `tqhnsw.disable_binary_prefilter` is direct evidence the prefilter
  is actually live, not just declared live. This is the right level
  of paranoia for a baseline packet.

### Concerns

1. **Warm SQL cell is `50` queries, `3` prime passes.** Tight by
   design but means the p99 row (`7.952ms`) is a single outlier. Fine
   for a baseline shape; just don't quote p99 as a comparison point
   in later packets without widening the query set.
2. **Traversal residual is derived by subtraction.** `3.05ms`
   residual = `amrescan_total - binary_prefilter - exact_score -
   rerank`. Any future helper-timer miss shows up in "traversal."
   Worth a follow-up to instrument layer-search directly if a later
   packet wants to attribute traversal-cost changes.
3. **Doesn't call the task-text update explicitly.** The readout
   correctly says the task scope is partially invalidated, but
   `plan/tasks/16-turboquant-iteration.md` was not updated in this
   packet to reflect that. Fine to defer until a lever lands, but
   track it.

### Call

Accepted. This is exactly the baseline packet that should precede
lever work — code-backed, scope-aware, and short enough to read in
one sitting. Good foundation for packets `424`+.
