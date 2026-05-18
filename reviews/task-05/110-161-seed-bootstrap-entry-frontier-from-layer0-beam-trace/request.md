# Request: Seed Bootstrap Entry Frontier From Layer-0 Beam Trace

Commit: `2643ade`

Summary:
- make `scan.rs` consume the graph-owned layer-0 beam runner during bootstrap entry seeding when `ef_search > 1`
- seed visible frontier, visited state, expanded-source state, and scheduler state from the resulting beam trace
- keep the old single-entry path for `ef_search == 1` so refill-on-consume semantics stay unchanged there

Please review:
- whether entry seeding is the right first runtime seam for consuming the graph-owned beam trace
- whether keeping the `ef_search == 1` fast path on the old behavior is the right compatibility boundary for now
- whether seeding visible frontier plus scheduler state from the beam trace is the right next step toward replacing the old bootstrap refill loop
