# Request: Refill Bootstrap Frontier From Layer-0 Beam Trace

Commit: `24d3021`

Summary:
- make `scan.rs` use the graph-owned layer-0 beam runner for bootstrap refill after a consumed source
- keep the refill traversal depth at one expansion (`ef_search = 1`) so the existing bootstrap-after-success contract stays intact
- seed successor candidates from the resulting beam trace frontier instead of open-coding direct successor loading

Please review:
- whether this one-step beam-trace refill is the right compatibility bridge before deeper runtime traversal changes
- whether using a synthetic seed score for the single-source trace is acceptable at this seam
- whether the runtime is now using the graph-owned beam runner in the right two places: entry seeding and post-consume refill
