# Review Request: Task 144 Packet 004 - Probe Distance Ratio GUC

## Summary

This packet lands the Task 144 Phase 2 query-time pruning switch behind a default-off GUC:

- Adds `ec_spire.probe_distance_ratio` with default `0.0` meaning disabled.
- Carries the resolved ratio as micro-units through `SpireRecursiveRouteBudget`.
- Applies pruning after recursive leaf route selection and before scan diagnostics report the final deduped route count.
- Reports `truncation_reason = "probe_distance_ratio"` when the ratio gate removes routed leaves.
- Adds ADR-084 to pin the shared Task 144 closure/pruning distance surrogate to the current SPIRE route-score proxy:
  `d_route(score) = max(0, 1 - score)`.

The ADR explicitly carries the reviewer caveat from packet 002: this surrogate is order-preserving for current inner-product routing but is not norm-robust, and ratio bands can collapse when the best distance floors to zero. This slice does not promote the knob; it only makes the default-off query-side mechanism measurable.

## Code Under Review

- `a7b94ca0b2d2168a0972e1c2c4c28949c849488d` - `bench: add SPIRE probe distance ratio pruning`

Key files:

- `spec/adr/ADR-084-spire-closure-pruning-distance-surrogate.md`
- `src/am/ec_spire/options/mod.rs`
- `src/am/ec_spire/scan/routing.rs`
- `src/am/ec_spire/scan/types.rs`
- `src/am/ec_spire/options/tests.rs`
- `src/am/ec_spire/scan/tests/routing.rs`

## Validation

Packet-local log:

- `artifacts/cargo-test-probe-distance-ratio.log`

Command:

```text
script -q -c "cargo test -p ecaz probe_distance_ratio --no-default-features --features pg18" reviews/task-144/004-probe-distance-ratio-guc/artifacts/cargo-test-probe-distance-ratio.log
```

Result:

```text
running 2 tests
test am::ec_spire::options::tests::recursive_route_budget_carries_probe_distance_ratio ... ok
test am::ec_spire::scan::tests::route_recursive_routing_objects_to_leaf_routes_applies_probe_distance_ratio ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2264 filtered out; finished in 0.00s
```

## Limits / Next Work

This is not Task 144 closeout. No release benchmark matrix was run in this packet. The closeout matrix still needs built closure assignment plus ratio pruning measured via `ecaz bench suite` on release PG18 at 10k / 50k / 100k with recall, latency, row-fraction scanned, storage, probed-list distributions, and recall tails.
