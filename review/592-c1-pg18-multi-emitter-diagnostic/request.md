# PG18 multi-emitter diagnostic gate

## Summary

This packet covers the diagnostic slice in commit `f8e34609125c66f99b2b1b6e66c8372c44dc0e4b`.

The branch now has a `pg_test`/unit-test-only env gate:

```text
TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1
```

When set, attached parallel scan backends bypass the current single-emitter election and may all attempt to emit tuples. Production-style builds without `pg_test` keep returning `false` from the diagnostic gate, and the default PG18 runtime path still uses one elected tuple emitter.

## Result

Default PG18 execution remains serial-equivalent in both leader-participating and worker-only lanes:

```text
serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
planner-visible Parallel Index Scan validation passed
```

The diagnostic multi-emitter opt-in still launches the same real PG18 `Gather Merge -> Parallel Index Scan` plan, but it fails serial-equivalence in both lanes:

```text
leader on candidate_ids=[379, 93, 177, 472, 473, 378, 176, 71, 172, 280, 57, 366, 258, 82, 78, 459]
leader off candidate_ids=[379, 177, 473, 472, 378, 93, 57, 366, 258, 172, 280, 176, 71, 82, 459, 284]
```

This confirms the remaining blocker is still rank-compatible multi-emitter `Gather Merge` output, not PG18 worker launch or planner path selection.

## Artifacts

- `artifacts/pg18-parallel-multi-emitter-default.log`
- `artifacts/pg18-parallel-multi-emitter-default-leader-off.log`
- `artifacts/pg18-parallel-multi-emitter-enabled.log`
- `artifacts/pg18-parallel-multi-emitter-enabled-leader-off.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test parallel_scan_backend_may_emit_tuples --lib`
- `cargo test bind_parallel_scan_state_captures_shared_rescan_epoch --lib`
- `cargo test try_take_parallel_scan_next_output_suppresses_shared_emitted_heap_tid --lib`
- `cargo test --lib`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-multi-emitter-diagnostic-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --disable-parallel-leader-participation --log-output target/pg18-parallel-multi-emitter-diagnostic-default-leader-off.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-multi-emitter-diagnostic-enabled.log` (expected validation failure)
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --disable-parallel-leader-participation --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-multi-emitter-diagnostic-enabled-leader-off.log` (expected validation failure)
- `cargo test`
- `cargo pgrx test pg18`
- `git diff --check`
