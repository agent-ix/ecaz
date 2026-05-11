---
head_sha: 853d2ad6f705dfa8c857371f5703fc5a93a69121
packet: 30789-spire-stage-e-lifecycle-drop-before-fanout
timestamp: 2026-05-11T00:47:55Z
---

# Artifact Manifest

- Packet/topic: `30789-spire-stage-e-lifecycle-drop-before-fanout`
- Lane: Phase 11 Stage E local multi-instance lifecycle matrix
- Fixture: `drop_remote_index_before_fanout`
- Storage format: RaBitQ remote and coordinator indexes
- Rerank mode: compact candidate receive only; remote heap remains next stage
- Surface: one coordinator PG18 cluster plus one remote PG18 cluster addressed
  through two remote descriptor entries
- Isolated one-index-per-table vs shared-table: isolated fixture tables
- Head SHA: `853d2ad6f705dfa8c857371f5703fc5a93a69121`

## Command

```bash
cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 \
  --case drop_remote_index_before_fanout \
  --artifact-dir review/30789-spire-stage-e-lifecycle-drop-before-fanout/artifacts \
  --run-id 30789
```

## Artifacts

- `stage_e_lifecycle_drop_remote_index_before_fanout.log`
  - Key line: `stage_e_lifecycle_drop_remote_index_before_fanout_passed=true`
- `stage_e_lifecycle_drop_remote_index_before_fanout_strict.log`
  - Lifecycle row:
    `drop_remote_index_before_fanout,DROP INDEX,before_fanout_planning,fail_closed,remote_candidate_receive_failed,skip_node,degraded_skipped,remote_index_unavailable,compact_candidate_receive`
  - Injection: `DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx before fanout`
  - Key raw rows:
    `2,remote_candidate_receive_failed,remote_index_unavailable,0` and
    `3,ready,none,1`
  - Key summary:
    `spire_remote_fanout_executor_v1,2,2,1,1,remote_index_unavailable,1,0,none,compact_candidate_receive,remote_candidate_receive_failed`
- `stage_e_lifecycle_drop_remote_index_before_fanout_degraded.log`
  - Lifecycle row:
    `drop_remote_index_before_fanout,DROP INDEX,before_fanout_planning,fail_closed,remote_candidate_receive_failed,skip_node,degraded_skipped,remote_index_unavailable,compact_candidate_receive`
  - Injection: `DROP INDEX ec_spire_stage_e_lifecycle_dropped_idx before fanout`
  - Key raw rows:
    `2,remote_candidate_receive_failed,remote_index_unavailable,0` and
    `3,ready,none,1`
  - Key summary:
    `spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_index_unavailable,remote_heap_resolution,degraded_ready`
- `remote-ready-postgres.log`
  - Remote PostgreSQL server log for the fixture.
- `coord-postgres.log`
  - Coordinator PostgreSQL server log for the fixture.

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh`
- `cargo fmt --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`
