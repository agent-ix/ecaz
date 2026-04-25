# PG18 Contributor Diagnostic Path

## Summary

This packet covers commit `753efb11e3b67f90eb776ee8fb623375e8765325`.

The runtime now has a diagnostic-only PG18 contributor mode:

```text
TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1
```

When enabled, non-elected workers do not immediately mark themselves exhausted. They publish their current result into the hidden coordinator slot and briefly poll for the elected visible emitter to drain it. The default path remains the existing single elected visible tuple emitter.

## Result

The default lane is unchanged and still passes:

```text
Limit (actual time=14.456..15.169 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

The contributor diagnostic lane also preserves serial equivalence, but it does not improve work contribution yet:

```text
Limit (actual time=42.227..43.477 rows=16.00 loops=1)
Bootstrap Expansions: 17
Elements Scored: 17
Heap TIDs Returned: 16
Parallel Handoffs: Foreign Selected: 0
[pg18-parallel] candidate_missing_serial_ids=[]
[pg18-parallel] candidate_extra_ids=[]
[pg18-parallel] planner-visible Parallel Index Scan validation passed
```

Interpretation: the hidden-slot protocol is safe under the strict PG18 fixture, but the non-elected workers are currently publishing duplicate initial graph cursors. The next performance step is partitioned or otherwise distinct contributor work, so the elected emitter has useful foreign hidden rows to drain.

## Artifacts

- `artifacts/pg18-parallel-contributor-default.log`
- `artifacts/pg18-parallel-contributor-diagnostic.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo test contribute --lib`
- `cargo test contributor --lib`
- `cargo test`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --features pg_test --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-contributor-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --env TQVECTOR_PG18_PARALLEL_CONTRIBUTOR_DIAGNOSTIC=1 --log-output target/pg18-parallel-contributor-diagnostic.log`
- `git diff --check`
