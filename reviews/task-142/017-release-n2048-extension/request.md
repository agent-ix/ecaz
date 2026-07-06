# Task 142 Packet 017: Release n2048 Extension

Please review the Task 142 follow-up for the two open packet 016 findings:

1. Add `nlists=2048` release cells at 50k and 100k.
2. Add an explicit epoch-change invalidation regression for the routing hierarchy cache.

## Summary

The suite completed both missing release cells through `ecaz bench suite`:

- `release-50k-n2048-b0`
- `release-100k-n2048-b0`

Both cells installed a release `ecaz.so`, recorded release profiles for the
coordinator and all three remotes, and stamped every cited `spire-pipeline` row
with `backend_build_profile=release` and per-node release profiles.

## Key Results

Representative query rows:

```text
50k n2048/b0  nprobe 32  p50 67.854 ms  p95 71.188 ms  recall@10 0.8550
50k n2048/b0  nprobe 96  p50 67.143 ms  p95 68.716 ms  recall@10 0.9405
100k n2048/b0 nprobe 32  p50 66.869 ms  p95 69.309 ms  recall@10 0.8185
100k n2048/b0 nprobe 96  p50 69.311 ms  p95 73.781 ms  recall@10 0.9175
```

Steady-state profile rows keep the redundant-load staircase closed:

```text
50k n2048/b0  nprobe 32  total_p50 37.582 ms  manifest_load 0.023 ms  leaf_count 1.195 ms  route_select 4.435 ms  routing_loads 0
100k n2048/b0 nprobe 32  total_p50 37.445 ms  manifest_load 0.023 ms  leaf_count 1.181 ms  route_select 4.407 ms  routing_loads 0
```

Compared with packet 016 n1024 rows, the true route-select descent roughly
doubles from ~2.13 ms to ~4.40 ms, while `manifest_cache_hit_sum=200`,
`manifest_cache_miss_sum=0`, `routing_hierarchy_load_sum=0`, `socket_open_sum=0`,
and `endpoint_identity_query_sum=0` hold for every n2048 row.

The epoch-change regression passed:

```text
collect_cached_resolved_scan_plan_selection_reloads_on_epoch_change ... ok
```

## Review Focus

1. Confirm this packet closes packet 016 Finding 1 by adding 50k/100k `nlists=2048` release evidence.
2. Confirm the new regression closes packet 016 Finding 2 for epoch-keyed cache invalidation.
3. Confirm the interpretation is acceptable: no Task 141 pre-cache n2048 before-row exists, but the n2048 steady-state rows directly show the cache invariant holds and only the expected true routing descent scales.

Packet manifest: `artifacts/manifest.md`.
