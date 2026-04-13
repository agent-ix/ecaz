# Review Request: C1 ADR-032 Fused Node Cache Cut

## Context

Retrospective split from the original packet `293`.

Packets `291` and `292` showed that isolated element-cache and neighbor-cache arena substitutions
were both losing variants. This cut tried the first broader fused-node experiment after those two
weak variants.

## Attempt

- remove the separate scan-local neighbor cache
- remove the separate exact-score cache
- attach lazy neighbors and exact-score-once state directly to `CachedGraphElement`
- keep the rest of traversal keyed by tids

This was broader than `291`/`292`, but it still stopped short of the actual ADR-032 target because
frontier and result bookkeeping were still tid-keyed rather than slot-keyed.

## Validation

Green before benchmarking:

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Measurements

Canonical warm real-`50k`, release build, `m=8`, `ef_search=40`, `warmup-passes=3`,
`session-mode=per-cell`, `timing-mode=cached-plan`.

Standing kept ADR-031 Tier 1 baseline:

- `mean ~= 1.507-1.510ms`

All known runs for this attempt:

- run 1: `p50=1.560ms`, `p99=2.525ms`, `mean=1.588ms`
- run 2: `p50=1.564ms`, `p99=2.720ms`, `mean=1.602ms`

No other runs were retained for this attempt.

## Outcome

Discarded.

This cut did co-locate more node-local state than `291`/`292`, but it still left the scan
algorithm operating on tids rather than stable node slots. It did not remove enough repeated
lookup/join work to justify the larger fused object shape.
