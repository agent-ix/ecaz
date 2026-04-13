# Review Request: C1 ADR-032 Slot Frontier Scheduler Cut

## Context

Retrospective split from the original packet `293`.

After `294` showed that a fused node cache without slot-based traversal was still a losing cut,
this packet records the first real slot-based ADR-032 traversal attempt.

## Attempt

- introduce a scan-local `ScanNodeArena`
- keep one cached node object per `element_tid`
- move frontier and bootstrap scheduler storage to node-slot ids internally
- keep debug/readout helpers projecting slots back to tids so the review surface stayed readable

This was the first slice that actually exercised the intended ADR-032 slot-based architecture.

## Validation

Green before benchmarking:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Measurements

Canonical warm real-`50k`, release build, `m=8`, `ef_search=40`, `warmup-passes=3`,
`session-mode=per-cell`, `timing-mode=cached-plan`.

Standing kept ADR-031 Tier 1 baseline:

- `p50 ~= 1.480-1.485ms`
- `mean ~= 1.507-1.510ms`

All known runs for this attempt:

- run 1: `p50=1.548ms`, `p99=2.636ms`, `mean=1.592ms`
- run 2: `p50=1.542ms`, `p99=2.623ms`, `mean=1.579ms`

No other runs were retained for this attempt.

## Outcome

Discarded.

This was stronger than `294` because it really did move frontier and scheduler state onto stable
node slots. But merely changing the addressing scheme did not reduce the work per step enough to
beat the kept ADR-031 path.
