# Validation

- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo
  check --offline --all-targets --no-default-features --features pg18` passed
  after the marker regression assertion was added.
- `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config
  cargo check --offline --all-targets --no-default-features
  --features pg18,distann-head-attribution-benchmark` passed.
- The focused PG18 cache test passed:
  `am::ec_distann::head_cache::tests::cache_eviction_removes_oldest_matching_index`.
- The physical lifecycle test now asserts the persisted marker surface;
  compile validation passed for that assertion. A full pgrx lifecycle run was
  not repeated after the feature benchmark because the benchmark cluster was
  already shut down.
- Suite audit passed for all three steps:
  `ecaz bench suite audit --config artifacts/task206-feature-seed-ab.json`.
- The completed run used `ecaz bench suite run ...` and wrote
  `artifacts/run/results.jsonl` and `artifacts/run/suite-manifest.json`.

The physical logs record effective `head_seed_count=128` and `200` with
distinct seed digests at 10k, 50k, and 100k. They also record physical-path
`ec_distann_scan_round` notices with numeric transport wait, straggler spread,
request bytes, and response bytes; `expanded_nodes=unmeasured` is explicit
because that quantity is not measured on this path.

The default/uninstrumented build now emits `absent` for attribution fields,
rather than presenting unavailable values as zero. The cache fix removes the
oldest matching index entry, covered by the focused regression test.
