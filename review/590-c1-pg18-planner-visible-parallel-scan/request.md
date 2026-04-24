# PG18 planner-visible parallel scan

## Summary

This packet covers the PG18 activation checkpoint for real planner-visible `Parallel Index Scan` paths on `ec_hnsw`.

The branch now:

- enables `amcanparallel` for PG18 builds,
- models graph work as run cost and discounts completed partial `ec_hnsw` index paths after PG assigns workers,
- keeps PG18 scan ORDER BY output storage stable for executor lifetime,
- elects a single tuple-emitting backend per parallel scan epoch while non-emitters exhaust before publishing local candidates,
- preserves serial-equivalent ordered output for both leader-participating and worker-only execution.

The current runtime posture is intentionally one elected tuple emitter. Rank-compatible multi-emitter `Gather Merge` output remains the next runtime step.

## Result

Both final PG18 release validation runs chose and executed:

```text
Limit
  -> Gather Merge
       Workers Planned: 4
       Workers Launched: 4
       -> Parallel Index Scan using pg18_parallel_scan_fixture_idx
```

The ordered candidate IDs matched serial IDs in both lanes:

```text
[177, 379, 472, 473, 378, 172, 93, 280, 57, 366, 258, 176, 82, 71, 459, 284]
```

Planner diagnostics reported:

```text
partial_ec_hnsw_index_path_count=1
best_partial_ec_hnsw startup_cost=0.000 total_cost=1021.159 parallel_workers=4 parallel_aware=true pathkeys=1
```

## Artifacts

- `artifacts/pg18-parallel-exec-default.log`
- `artifacts/pg18-parallel-exec-default-leader-off.log`
- `artifacts/manifest.md`

## Validation

- `cargo fmt`
- `cargo check`
- `cargo check -p ecaz-cli`
- `cargo test --lib`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features 'pg18 pg_test' --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --log-output target/pg18-parallel-exec-default.log`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --expect-parallel --diagnose-planner --disable-parallel-leader-participation --log-output target/pg18-parallel-exec-default-leader-off.log`

