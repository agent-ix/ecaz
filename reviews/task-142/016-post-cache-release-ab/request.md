# Task 142 Packet 016: Post-Cache Release A/B Anchors

Please review the Task 142 post-cache release benchmark packet.

## Summary

This packet runs the release local-multinode anchors after the Task 142
production-read cache work and the packet 015 microsecond timer fix.

Evidence is via `ecaz bench suite` only. The first four-cell suite completed
10k n128, 50k n128, and 50k n1024, then hit host disk exhaustion during the
100k n1024 coordinator load. After deleting generated target run directories,
I reran only `release-100k-n1024-b0`; that retry passed and produced the
accepted 100k result.

## Key Results

Packet manifest: `artifacts/manifest.md`.

All cited cells installed a release `ecaz.so`, recorded release
`node_build_profile` lines for coordinator and all three remotes, and have
nested `bench-suite/results.jsonl` rows stamped with:

```text
backend_build_profile=release
backend_node_profiles=coordinator:39800:release,local-port-39801:39801:release,local-port-39802:39802:release,local-port-39803:39803:release
```

Matched Task 141 release anchor p50 deltas:

- 50k n128/b0: query p50 improved 8.2% to 11.0% across nprobe 8/16/32/64/96; recall unchanged.
- 50k n1024/b0: query p50 improved 40.8% to 44.5%; recall unchanged.
- 100k n1024/b0: query p50 improved 39.2% to 42.5%; recall unchanged.

Representative accepted rows:

```text
50k n128/b0  nprobe 32  p50 62.769 ms  p95 65.747 ms  recall@10 0.9725
50k n1024/b0 nprobe 32  p50 61.402 ms  p95 62.854 ms  recall@10 0.8895
100k n1024/b0 nprobe 32 p50 62.392 ms  p95 65.062 ms  recall@10 0.8615
```

The profile rows show warm-cache behavior for the production-read path:
`manifest_cache_hit_sum=200`, `routing_hierarchy_load_sum=0` for every cited
nprobe row, with connection-pool hits at or near the full remote dispatch
count.

## Review Focus

1. Confirm packet 016 is acceptable release A/B evidence against Task 141 for
   the 10k/50k/100k anchor subset.
2. Confirm the retry handling is documented enough: initial 100k disk failure
   is preserved in `suite-run.log`; accepted 100k evidence is in
   `suite-run-100k-retry.log` and `suite-manifest-100k-retry.json`.
3. Confirm the profile rows and backend provenance satisfy the Task 141/142
   release-guard expectations before continuing to the next Task 142 slice.
