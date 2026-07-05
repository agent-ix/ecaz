# Task 142 Packet 014: Release Multinode Smoke

## Summary

This packet records a release-profile SPIRE local multinode smoke run for the
Task 142 cache/profile work. It validates that the local multinode fixture can
install and run release `ecaz.so`, and that the suite evidence captures the
backend build profile for the coordinator and every remote node.

Scope note: this is substrate/profile evidence for the release benchmark lane,
not the final Task 142 closeout A/B matrix.

## Validation

- `target/release/ecaz bench suite run --config artifacts/task142-release-10k-n128-smoke-suite.json ...`
  - log: `artifacts/suite-run.log`
  - result: `SPIRE local multinode fixture passed`; `HARNESS PASSED`
  - release evidence: `install_profile=release`; all four
    `node_build_profile` lines report `profile=release`
- Nested production-read suite:
  - manifest: `artifacts/release-10k-n128-b0-smoke/bench-suite/suite-manifest.json`
  - results: `artifacts/release-10k-n128-b0-smoke/bench-suite/results.jsonl`
  - `spire-pipeline` rows include `backend_build_profile=release` and
    `backend_node_profiles=coordinator:39700:release,local-port-39701:39701:release,local-port-39702:39702:release,local-port-39703:39703:release`

## Key Results

- Fixture: 10k real corpus, `nlists=128`, `boundary_replica_count=0`,
  `storage_format=rabitq`, top-k 10, nprobe sweep 8/16/32, 20 queries.
- Recall/latency smoke results:
  - nprobe 8: recall@k 0.9650, latency p50 58.927 ms
  - nprobe 16: recall@k 0.9800, latency p50 58.863 ms
  - nprobe 32: recall@k 0.9950, latency p50 59.546 ms
- Warm production profile counters show the Task 142 caches in use:
  - `manifest_cache_hit_sum=20`, `manifest_cache_miss_sum=0`
  - `connection_pool_hit_sum=58/60/60`, `connection_pool_miss_sum=0`
  - `socket_open_sum=0`, `routing_hierarchy_load_sum=0`

## Review Notes

Generated corpus shard TSVs under the artifact tree are intentionally left
untracked per `AGENTS.md`. The committed packet keeps the suite configs,
manifests, result JSONL, and cited logs only.
