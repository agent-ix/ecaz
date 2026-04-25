# ADR-040 Gather Merge Compatibility Amendment

## Summary

This packet covers commit `e4e97d51e6d17116e5f8587c052e49a4d1148467`.

ADR-040 now records the PG18 `Gather Merge` compatibility constraint discovered while testing diagnostic multi-emitter output:

- production PG18 parallel index scans preserve strict serial equivalence by electing one tuple emitter;
- direct multi-emitter output remains diagnostic-only;
- a future multi-emitter design needs rank-compatible sequencing, exact-key-sorted streams, or a planner/runtime contract that avoids `Gather Merge`.

## Evidence

The score-diagnostic logs show that the serial HNSW order can contain adjacent inversions under the exact SQL `ORDER BY` expression:

```text
serial_exact_score_adjacent_inversions=[177(-1.646769881) before 379(-1.742236257), 472(-1.641386509) before 473(-1.769334435), 172(-1.507030725) before 93(-1.649150252), 57(-1.543027639) before 366(-1.578914285), 258(-1.510545969) before 176(-1.518822074), 82(-1.491723895) before 71(-1.584297538), 459(-1.478064418) before 284(-1.587190151)]
```

Default elected-emitter PG18 execution still passes serial equivalence:

```text
serial_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
candidate_ids=[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
planner-visible Parallel Index Scan validation passed
```

Diagnostic multi-emitter output still fails strict serial equivalence:

```text
candidate_ids=[473, 379, 379, 93, 177, 472, 378, 280, 366, 92, 258, 176, 172, 57, 366, 82]
validation failed
```

## Artifacts

- `artifacts/pg18-parallel-score-diagnostic-default.log`
- `artifacts/pg18-parallel-score-diagnostic-multi-emitter.log`
- `artifacts/manifest.md`

## Validation

- `git diff --check`
- `cargo check -p ecaz-cli`
- `cargo test -p ecaz-cli`
- `cargo clippy -p ecaz-cli --all-targets -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-score-diagnostic-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_MULTI_EMITTER_DIAGNOSTIC=1 --log-output target/pg18-parallel-score-diagnostic-multi-emitter.log` (expected validation failure)
