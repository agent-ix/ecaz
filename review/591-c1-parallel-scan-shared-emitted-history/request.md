# Parallel scan shared emitted history

## Summary

This packet covers the runtime groundwork slice in commit `d3357bd4caf85f0d0754b8597fd45b6459f4968f`.

The branch now adds coordinator-owned emitted heap-TID history for parallel scans:

- bumps the AM-private parallel scan DSM layout version to `15`,
- records emitted heap TIDs in a coordinator ring when a parallel admitted result is consumed,
- rejects selected pending outputs whose heap TID is already in that shared emitted history,
- checks shared emitted history before republishing active local or handoff output,
- clears the shared emitted history on parallel scan reset/rescan.

The current runtime posture remains intentionally one elected tuple emitter. This slice only closes a durable duplicate-ownership gap needed before another multi-emitter attempt.

## Result

The PG18 planner-visible executor validation still chooses and executes:

```text
Limit
  -> Gather Merge
       Workers Planned: 4
       Workers Launched: 4
       -> Parallel Index Scan using pg18_parallel_scan_fixture_idx
```

The ordered candidate IDs matched serial IDs in both leader-participating and worker-only lanes:

```text
[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
```

## Artifacts

- `artifacts/pg18-parallel-shared-emitted-default.log`
- `artifacts/pg18-parallel-shared-emitted-leader-off.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
- `cargo pgrx test pg18`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-shared-emitted-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --disable-parallel-leader-participation --log-output target/pg18-parallel-shared-emitted-leader-off.log`
