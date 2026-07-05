# Review Request: SPIRE Phase 13e Distributed Placement Output

## Summary

This checkpoint advances Task 30 Phase 13e.1 from config validation to a real
static distribution artifact path. `ecaz corpus load --profile ec_spire` can now
consume the distributed placement config, split corpus rows into deterministic
per-remote TSV files, and emit a JSON placement plan instead of falling back to
local-only coordinator loading.

Code commit: `13e3bc57b2ce2af7b9e39c2a83a1f3a2171349ef`

## Changes

- Added `--distributed-placement-output-dir`.
- `--distributed-placement-config` now requires the output directory and refuses
  local-only fallback.
- Non-chunked and chunked corpus inputs can be streamed into per-node output
  files under `node-{node_id}/`.
- Rows are assigned by deterministic SHA-256-derived source identity hash over
  `id`, modulo the configured shard count.
- The output plan records coordinator index name, shard policy/count, total
  rows, per-remote row counts, shard row counts, corpus file paths, remote index
  regclass values, and the required `EC_SPIRE_REMOTE_CONNINFO_*` provider keys.
- Validation keeps rejecting incomplete shard coverage, duplicate shards,
  local node IDs, and non-SPIRE profiles.

## Validation

See `artifacts/manifest.md`.

- `cargo test -p ecaz-cli distributed_placement`
- Result: 7 passed, 0 failed

## Scope Notes

This still does not complete remote PostgreSQL materialization, descriptor
registration, or coordinator placement publication. It does make AWS-scale data
distribution explicit and reproducible, and it prevents the previously observed
failure mode where a distributed run silently produced local-only placements.
