# Review 30780: SPIRE Stage E Epoch Mismatch

## Summary

This packet lands the `epoch_mismatch` Stage E fault row. It proves that a
remote descriptor whose served epoch window is stale blocks before socket
dispatch in strict mode, while degraded mode records the stale descriptor as
a single skipped dispatch and leaves the ready remote pending for production
transport.

Runtime command:

```text
cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 \
  --case epoch_mismatch \
  --artifact-dir review/30780-spire-stage-e-epoch-mismatch/artifacts \
  --run-id 30780
```

## Code Scope

- Extended `ecaz dev spire-multicluster fault-pg18` with
  `--case epoch_mismatch`.
- Extended `scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh`
  so the same pre-dispatch fixture family covers both version skew and stale
  epoch windows.
- Updated the Phase 11 task note with packet-local epoch-mismatch evidence.

## Runtime Evidence

Artifacts are under
`review/30780-spire-stage-e-epoch-mismatch/artifacts/`.

Strict mode:

```text
observed_summary=spire_remote_fanout_executor_v1,2,1,1,1,0,none,remote_epoch_window,stale_epoch
```

Interpreted columns:

```text
state_model,dispatch_count,planned_dispatch_count,blocked_before_dispatch_count,
transport_pending_dispatch_count,degraded_skipped_dispatch_count,
first_degraded_skip_category,next_executor_step,status
```

Strict mode therefore has two dispatch rows, one ready remote still planned,
one stale-epoch descriptor blocked before dispatch, zero degraded skips, and
status `stale_epoch`.

Degraded mode:

```text
observed_summary=spire_remote_fanout_executor_v1,2,2,0,1,1,stale_epoch,production_transport_adapter,requires_production_transport_adapter
```

Degraded mode has two dispatch rows, no remaining blocked-before-dispatch
rows, one degraded skip with category `stale_epoch`, and the ready remote
remains pending for production transport.

Pass signal:

```text
stage_e_fault_epoch_mismatch_passed=true
SPIRE Stage E epoch_mismatch PG18 fixture passed
```

## Validation

- `bash -n scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh`
- `cargo fmt --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- ...`
- Runtime fixture command shown above.

The shared pre-dispatch executor-state behavior was validated in packet 30779
with:

```text
cargo test production_executor_degraded_pre_dispatch_block_skips_node --no-default-features --features pg18
cargo check --no-default-features --features "pg18 pg_test"
```

## Review Questions

1. Is `last_served_epoch = 0` against requested epoch `1` an acceptable local
   epoch-mismatch injection for this pre-dispatch fixture row?
2. Is reusing the version-skew pre-dispatch fixture family the right shape for
   stale epoch windows?
3. Should the next Stage E row stay in pre-dispatch capability failures, or
   move into receive-time endpoint identity failures?
