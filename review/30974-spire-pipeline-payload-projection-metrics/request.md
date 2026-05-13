# Review Request: SPIRE Pipeline Payload Projection Metrics

## Summary

Please review commit `ef0926b8` (`Add SPIRE pipeline payload projection metrics`).

This is a harness slice for the remaining Phase 12.2 tuple-heavy throughput
measurement row. The existing `ecaz bench spire-pipeline` query-metrics path
timed id-only coordinator KNN scans, so `--remote-tuple-transport` could select
JSON versus typed transport without necessarily forcing payload slot delivery.

This slice adds:

- `--query-metric-projection-columns`, a comma/repeatable option for extra
  corpus columns to project during query latency measurement;
- identifier validation for those column names;
- SQL generation that always keeps `id` as the first selected column for recall
  accounting, then appends unique projected payload columns;
- benchmark header output recording the active projection shape;
- tracker evidence that the benchmark can now time payload-projection scans
  without editing fixture scripts.

The Phase 12.2 measurement row remains open until JSON versus
`pg_binary_attr_v1` live artifacts are captured packet-locally.

## Validation

- `cargo test -p ecaz-cli spire_pipeline`
  - passed: 13 passed, 0 failed.
- `cargo fmt --check`
  - passed; rustfmt emits existing nightly-only config warnings.
- `git diff --check`
  - passed.

## Requested Review

Please focus on:

1. Whether projecting additional columns in the query-metrics KNN SQL is the
   right harness boundary for tuple-heavy read measurement.
2. Whether keeping `id` first is sufficient for existing recall accounting.
3. Whether the tracker wording correctly leaves the actual throughput
   measurement row open until packet-local JSON-vs-typed artifacts exist.
