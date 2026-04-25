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

Follow-up diagnostics from commit `1f64c1acda03b5c8d59699f82e66df6038697f05` also record exact SQL `ORDER BY` scores for the serial and candidate IDs. The default elected-emitter path remains serial-equivalent, but the serial HNSW order itself contains adjacent exact-score inversions:

```text
serial_exact_score_adjacent_inversions=[177(-1.646769881) before 379(-1.742236257), 472(-1.641386509) before 473(-1.769334435), 172(-1.507030725) before 93(-1.649150252), 57(-1.543027639) before 366(-1.578914285), 258(-1.510545969) before 176(-1.518822074), 82(-1.491723895) before 71(-1.584297538), 459(-1.478064418) before 284(-1.587190151)]
```

That is the concrete `Gather Merge` blocker: if multiple workers expose these serial-order rows on separate streams, `Gather Merge` compares the exact sort key across stream heads and can reorder rows away from the single-emitter serial order.

## Artifacts

- `artifacts/pg18-parallel-snapshot-blocker-default.log`
- `artifacts/pg18-parallel-snapshot-blocker-diagnostic.log`
- `artifacts/pg18-parallel-score-diagnostic-default.log`
- `artifacts/pg18-parallel-score-diagnostic-multi-emitter.log`
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
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-score-diagnostic-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-score-diagnostic-multi-emitter.log` (expected validation failure)
- `git diff --check`
