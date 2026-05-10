# Review 30778: SPIRE Stage E Simulated Network Partition

## Summary

This packet lands the first Stage E fault-matrix runtime fixture:
`simulated_network_partition`. It proves the strict/degraded policy split
against a one-coordinator, one-ready-remote, one-unreachable-conninfo PG18
fixture through the `ecaz` operator path:

```text
target/debug/ecaz dev spire-multicluster fault-pg18 \
  --case simulated_network_partition \
  --artifact-dir review/30778-spire-stage-e-network-partition/artifacts \
  --run-id 30778e
```

The fixture uses the production async libpq transport adapter via a pg_test
summary helper, but stops at transport summary. It intentionally does not
claim heap row materialization or rerank coverage.

## Code Scope

- Added `ecaz dev spire-multicluster fault-pg18 --case simulated_network_partition`.
- Added `scripts/run_spire_multicluster_stage_e_network_partition_pg18.sh`.
- Added pg_test helpers for:
  - env-backed transport probe summary over the production adapter;
  - atomic multi-pid placement rewrites for the fixture.
- Updated the Phase 11 task note to mark the first Stage E fault row logged.

## Runtime Evidence

Artifacts are packet-local under
`review/30778-spire-stage-e-network-partition/artifacts/`.

Strict mode:

```text
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,connect_failed,1,0,none,production_transport_adapter,remote_transport_failed
```

This means two dispatches were attempted, one ready remote completed, one
unreachable conninfo failed with `connect_failed`, no degraded skip was
applied, and the status is `remote_transport_failed`.

Degraded mode:

```text
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,connect_failed,compact_candidate_receive,requires_compact_candidate_receive
```

This means the unreachable conninfo was skipped before transport send, the
ready remote still completed, the degraded skip category is `connect_failed`,
and the next executor step is `compact_candidate_receive`.

Pass signal:

```text
stage_e_fault_simulated_network_partition_passed=true
SPIRE Stage E simulated network partition PG18 fixture passed
```

## Validation

- `cargo fmt --check`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `bash -n scripts/run_spire_multicluster_stage_e_network_partition_pg18.sh`
- `git diff --check -- ...`
- `cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 --help`
- Runtime fixture command shown above.

## Review Questions

1. Is `fault-pg18 --case simulated_network_partition` the right CLI shape for
   extending the rest of packet 30770's fault matrix?
2. Does the transport-summary fixture draw the right boundary for this first
   row, or should the next row move closer to full operator/heap
   materialization?
3. Are the strict/degraded counters strong enough to prevent regressions in
   fail-closed versus skip-node policy?
