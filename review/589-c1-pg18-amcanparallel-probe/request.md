# 589-c1 PG18 amcanparallel Local Probe

## Request

Review the local-only PG18 `amcanparallel` probe captured from head
`e83b8f93f060cc4f5515567420242f3fdfb634dc` plus this temporary uncommitted
one-line change:

```rust
amroutine.amcanparallel = cfg!(feature = "pg18");
```

The source change was not committed. After the probe, I restored
`amroutine.amcanparallel = false;` and reinstalled the normal PG18 build.

## Result

The pathlist hook now answers the next activation question. With
`amcanparallel` temporarily advertised on PG18, PostgreSQL does generate a
partial `ec_hnsw` index path for the ordered query:

- `amcanparallel_seen=true`
- `partial_path_count=1`
- `partial_index_path_count=1`
- `partial_ec_hnsw_index_path_count=1`
- `best_partial_ec_hnsw ... parallel_workers=4 parallel_aware=true pathkeys=1`

The final plan still remains a serial ordered `Index Scan`, so the blocker is no
longer path generation once `amcanparallel` is advertised. It is cost/path
selection: the partial path has the same startup-heavy AM cost as the serial
path, and the serial `Limit -> Index Scan` remains selected.

The probe also preserves correctness on the fixture:

- serial and candidate ordered IDs match exactly
- the worker-control seqscan still launches 4 workers

## Artifacts

- `artifacts/pg18-amcanparallel-probe.log`
- `artifacts/manifest.md`

## Validation

- Temporary local source probe: `amroutine.amcanparallel = cfg!(feature = "pg18");`
- `cargo pgrx install --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --features pg18 --no-default-features`
- `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --diagnose-planner --log-output review/589-c1-pg18-amcanparallel-probe/artifacts/pg18-amcanparallel-probe.log`
- Restored `amroutine.amcanparallel = false;`
- Reinstalled normal PG18 build with the same `cargo pgrx install ... --features pg18 --no-default-features` command
- Verified the restored installed build with `cargo run -p ecaz-cli -- dev test pg18-parallel-scan --diagnose-planner --log-output target/pg18-postprobe-normal.log`, which reported `amcanparallel_seen=false`

## Review Focus

- Does this prove the remaining blocker is cost/path selection after
  `amcanparallel` is enabled?
- Should the next code slice change the cost split so partial index paths get a
  modeled benefit before landing `amcanparallel=true`?
