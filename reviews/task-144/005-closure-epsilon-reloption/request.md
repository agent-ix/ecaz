# Review Request: Task 144 Packet 005 - Closure Epsilon Reloption

## Summary

This packet lands the Task 144 Phase 1 build-side closure switch behind a default-off reloption:

- Adds `closure_epsilon` to `ec_spire` reloptions, default `0.0`.
- Keeps current fixed-count boundary assignment behavior when `closure_epsilon = 0`.
- For `closure_epsilon > 0`, build routing selects leaf replicas whose ADR-084 route-score distance proxy is within `best_distance * (1 + closure_epsilon)`.
- Caps closure replicas with the existing `boundary_replica_count` reloption, so default options still produce primary-only assignment.
- Threads `closure_epsilon` through single-level and recursive build assignment planning.

This pairs with packet 004's default-off query-side `ec_spire.probe_distance_ratio` GUC. It does not yet run the required closeout matrix.

## Code Under Review

- `a04cb85cb4e18eb79391ab792c30b9ff1b85f450` - `bench: add SPIRE closure epsilon assignment`

Key files:

- `src/am/ec_spire/options/mod.rs`
- `src/am/ec_spire/build/routing_plan.rs`
- `src/am/ec_spire/build/drafts.rs`
- `src/am/ec_spire/build/recursive.rs`
- `src/am/ec_spire/build/types.rs`
- `src/am/ec_spire/build/tests/centroid_state.rs`
- `src/am/ec_spire/options/tests.rs`

## Validation

Packet-local log:

- `artifacts/cargo-test-closure.log`

Command:

```text
script -q -c "cargo test -p ecaz closure --no-default-features --features pg18" reviews/task-144/005-closure-epsilon-reloption/artifacts/cargo-test-closure.log
```

Result:

```text
running 2 tests
test am::ec_spire::options::tests::closure_epsilon_reloption_accepts_default_off_ratio_band ... ok
test am::ec_spire::build::tests::single_level_route_map_plans_closure_replica_pids_by_distance_ratio ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2266 filtered out; finished in 0.00s
```

## Limits / Next Work

This is not Task 144 closeout evidence. No release build, storage step, recall step, or latency step ran here.

Next required Task 144 work is a real release `ecaz bench suite` matrix over closure/pruning cells on top of the Task 143-fixed operating point, with 10k / 50k / 100k recall, latency, percent row-instances scanned, storage, probed-list distributions, and recall tails.
