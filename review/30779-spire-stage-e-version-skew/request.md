# Review 30779: SPIRE Stage E Version Skew

## Summary

This packet lands the `version_skew` Stage E fault row. It proves that an
incompatible remote descriptor extension version blocks before socket dispatch
in strict mode, while degraded mode records that blocked descriptor as a
single skipped dispatch and leaves the ready remote pending for production
transport.

Runtime command:

```text
target/debug/ecaz dev spire-multicluster fault-pg18 \
  --case version_skew \
  --artifact-dir review/30779-spire-stage-e-version-skew/artifacts \
  --run-id 30779b
```

## Code Scope

- Extended `ecaz dev spire-multicluster fault-pg18` with
  `--case version_skew`.
- Added `scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh`.
- Updated production executor-state composition so degraded mode converts
  blocked-before-dispatch remote rows into degraded skipped dispatches before
  aggregation.
- Added a focused Rust unit test for degraded pre-dispatch skip behavior.
- Updated the Phase 11 task note with the packet-local version-skew evidence.

## Runtime Evidence

Artifacts are under
`review/30779-spire-stage-e-version-skew/artifacts/`.

Strict mode:

```text
observed_summary=spire_remote_fanout_executor_v1,2,1,1,1,0,none,remote_extension_version,incompatible_extension_version
```

Interpreted columns:

```text
state_model,dispatch_count,planned_dispatch_count,blocked_before_dispatch_count,
transport_pending_dispatch_count,degraded_skipped_dispatch_count,
first_degraded_skip_category,next_executor_step,status
```

Strict mode therefore has two dispatch rows, one ready remote still planned,
one incompatible-version descriptor blocked before dispatch, zero degraded
skips, and status `incompatible_extension_version`.

Degraded mode:

```text
observed_summary=spire_remote_fanout_executor_v1,2,2,0,1,1,incompatible_extension_version,production_transport_adapter,requires_production_transport_adapter
```

Degraded mode has two dispatch rows, no remaining blocked-before-dispatch
rows, one degraded skip with category `incompatible_extension_version`, and
the ready remote remains pending for production transport.

Pass signal:

```text
stage_e_fault_version_skew_passed=true
SPIRE Stage E version_skew PG18 fixture passed
```

## Validation

- `cargo fmt --check`
- `cargo test production_executor_degraded_pre_dispatch_block_skips_node --no-default-features --features pg18`
- `cargo check --no-default-features --features "pg18 pg_test"`
- `bash -n scripts/run_spire_multicluster_stage_e_predispatch_fault_pg18.sh`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli spire_multicluster -- --nocapture`
- `git diff --check -- ...`
- `cargo run -p ecaz-cli -- dev spire-multicluster fault-pg18 --case version_skew --help`
- Runtime fixture command shown above.

## Review Questions

1. Is converting blocked-before-dispatch remote rows into degraded skipped
   dispatches the right executor-state model for pre-dispatch Stage E faults?
2. Does the version-skew fixture draw the correct boundary by stopping at
   production executor-state summary rather than opening sockets?
3. Should the same pre-dispatch script be extended next for `epoch_mismatch`,
   or should epoch freshness get its own lifecycle-oriented fixture?
