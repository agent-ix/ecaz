# Review Request: SPIRE Stage E Lifecycle Drop Before Fanout

## Summary

This packet adds the first Stage E lifecycle runtime fixture:
`drop_remote_index_before_fanout`.

The code checkpoint is `853d2ad6f705dfa8c857371f5703fc5a93a69121`.

## What Changed

- Added `ecaz dev spire-multicluster lifecycle-pg18`.
- Added `scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh`.
- The first lifecycle case creates remote and coordinator RaBitQ indexes, binds
  the planned remote identity, drops the remote index before fanout, and then
  drives production candidate receive.
- Updated Phase 11 to record packet `30789`.

## Evidence

Command:

```bash
cargo run -p ecaz-cli -- dev spire-multicluster lifecycle-pg18 \
  --case drop_remote_index_before_fanout \
  --artifact-dir review/30789-spire-stage-e-lifecycle-drop-before-fanout/artifacts \
  --run-id 30789
```

Strict mode:

```text
observed_candidate_receive_rows=2,remote_candidate_receive_failed,remote_index_unavailable,0
3,ready,none,1
observed_summary=spire_remote_fanout_executor_v1,2,2,1,1,remote_index_unavailable,1,0,none,compact_candidate_receive,remote_candidate_receive_failed
```

Degraded mode:

```text
observed_candidate_receive_rows=2,remote_candidate_receive_failed,remote_index_unavailable,0
3,ready,none,1
observed_summary=spire_remote_fanout_executor_v1,2,1,1,0,none,1,1,remote_index_unavailable,remote_heap_resolution,degraded_ready
```

Pass marker:

```text
stage_e_lifecycle_drop_remote_index_before_fanout_passed=true
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh`
- `cargo fmt --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- crates/ecaz-cli/src/commands/dev/spire_multicluster.rs crates/ecaz-cli/src/cli.rs scripts/run_spire_multicluster_stage_e_lifecycle_pg18.sh plan/tasks/task30-phase11-spire-distributed-production-parity.md`

## Review Focus

- Is this the right first lifecycle fixture shape before adding in-flight DDL
  timing cases?
- Does dropping before fanout correctly map to `remote_index_unavailable`
  through production candidate receive?
- Is `lifecycle-pg18` a good CLI split from `fault-pg18`?
