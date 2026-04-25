# PG18 diagnostic blocker snapshot

## Summary

This packet covers commit `b43a4bee3dee6703bddc922afb6592a02ffe9f46`.

The PG18 planner integration snapshot now reports a distinct `next_runtime_blocker` when the `pg_test`/unit-test-only diagnostic env is active:

```text
TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1
```

Default runs still report the one-elected-emitter blocker. Diagnostic runs now report that direct multi-emitter output remains rank-incompatible with `Gather Merge`.

## Result

Default PG18 execution remains serial-equivalent and keeps the normal blocker text:

```text
next_runtime_blocker=PG18 planner-visible Parallel Index Scan is enabled with one elected tuple emitter; rank-compatible multi-emitter Gather Merge output remains the next runtime step
serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
planner-visible Parallel Index Scan validation passed
```

The diagnostic multi-emitter run still launches real PG18 parallel index execution, fails serial-equivalence as expected, and now logs the diagnostic-specific blocker:

```text
next_runtime_blocker=PG18 diagnostic multi-emitter env is enabled; direct multi-emitter output remains rank-incompatible with Gather Merge
serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
candidate_ids=[473, 379, 93, 177, 472, 378, 280, 366, 57, 176, 258, 71, 172, 280, 176, 284]
validation failed
```

## Artifacts

- `artifacts/pg18-parallel-snapshot-blocker-default.log`
- `artifacts/pg18-parallel-snapshot-blocker-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo pgrx test pg18 test_ech_planner_integration_snapshot_reports`
- `cargo pgrx test pg18 test_ech_planner_snapshot_reports_multi_emitter_blocker`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx test pg18`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-snapshot-blocker-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-snapshot-blocker-diagnostic.log` (expected validation failure)
- `git diff --check`
