---
head_sha: 04ed2d9f0ba4038455d522f168ff64c5c3056c02
packet: 30788-spire-stage-e-remote-oom
timestamp: 2026-05-11T00:40:22Z
---

# Artifact Manifest

- Packet/topic: `30788-spire-stage-e-remote-oom`
- Lane: Phase 11 Stage E local multi-instance fault matrix
- Fixture: `remote_oom`
- Storage format: transport probe only; no index storage asserted
- Rerank mode: not applicable
- Surface: one coordinator PG18 cluster plus one remote PG18 cluster addressed
  through two remote descriptor entries
- Isolated one-index-per-table vs shared-table: not applicable; transport
  fault fixture does not build remote indexes
- Head SHA: `04ed2d9f0ba4038455d522f168ff64c5c3056c02`

## Command

```bash
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case remote_oom \
  --artifact-dir review/30788-spire-stage-e-remote-oom/artifacts \
  --run-id 30788
```

## Artifacts

- `stage_e_fault_remote_oom.log`
  - Key line: `stage_e_fault_remote_oom_passed=true`
- `stage_e_fault_remote_oom_strict.log`
  - Matrix row:
    `remote_oom,remote_query_failed,fail_closed,remote_transport_failed,skip_node,degraded_skipped,remote_query_failed_count+1; degraded_skipped_dispatch_count+1`
  - Key raw rows:
    `2,remote_transport_failed,remote_query_failed,0` and
    `3,ready,none,3`
  - Key summary:
    `spire_remote_fanout_executor_v1,2,2,1,1,remote_query_failed,1,0,none,production_transport_adapter,remote_transport_failed`
- `stage_e_fault_remote_oom_degraded.log`
  - Matrix row:
    `remote_oom,remote_query_failed,fail_closed,remote_transport_failed,skip_node,degraded_skipped,remote_query_failed_count+1; degraded_skipped_dispatch_count+1`
  - Key raw rows:
    `2,remote_transport_failed,remote_query_failed,0` and
    `3,ready,none,3`
  - Key summary:
    `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_query_failed,compact_candidate_receive,requires_compact_candidate_receive`
- `remote-ready-postgres.log`
  - Remote PostgreSQL server log for the fixture.
- `coord-postgres.log`
  - Coordinator PostgreSQL server log for the fixture.

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check --no-default-features --features pg18,pg_test`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- src/lib.rs crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_transport_fault_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
