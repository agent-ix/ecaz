# Task 29 Landing Readiness Refresh

## Request

Review the refreshed Task 29 landing recommendation after the Task 29b vacuum
consistency work and Task 29c build-performance profile.

Current branch head: `b0deb879dfd95eff0094929031015095da9473e2`

This packet supersedes the stale build-time conclusion in packet `11099`. The
raw logs cited here are copied into this packet's `artifacts/` directory.

## Summary

Task 29 is locally ready for outside landing review.

- Correctness smoke: focused PG18 DiskANN callback coverage passed
  `19 passed; 0 failed` in packet `11099`.
- Recall/latency: real-10k DiskANN with binary-sidecar prefilter and early-stop
  scan measured recall@10 `0.9955` at L=64, `0.9970` at L=200, and `0.9975`
  at L=800. L=200 latency measured mean/p50/p95/p99
  `58.5/55.9/75.0/90.1 ms`; L=800 measured `67.7/66.7/76.9/80.0 ms`.
- Storage: DiskANN measured `4.7 MiB` / `494.0 B` per row in the Task 29
  benchmark table. The refreshed release-mode Task 29c table measured
  `4,939,776` bytes (`4824 kB`).
- HNSW reference: earlier same-corpus `ec_hnsw` reference measured recall@10
  `0.9700`, mean latency `35.25 ms`, and p50/p95/p99
  `33.1/39.4/49.1 ms`, with `13.0 MiB` / `1366.4 B` per row. The Task 29c
  build reference on the same `task29c_phase_profile` table measured HNSW size
  `15,130,624` bytes (`14 MB`).
- Task 29b: vacuum repair now uses the same sidecar prefilter selection helper
  as scan. The isolated real-10k vacuum scenario measured pre-vacuum recall@10
  `0.9970` and post-vacuum live-row recall@10 `0.9975`.
- Task 29c: the previous `~492s` build-time concern was a debug/dev-installed
  extension artifact. With the release-installed extension, the same isolated
  real-10k DiskANN index-only build measured `79.238s`; HNSW with `m=32`,
  `ef_construction=100`, `build_source_column=source` built in `5.23s`.

## Recommendation

Land Task 29 / 29a / 29b / 29c as the initial DiskANN tuning slice.

The remaining performance gap is clear and scoped: release-mode DiskANN build is
still slower than HNSW on real-10k, but the cost is now known to be Vamana graph
construction, dominated by pass-1 greedy search, robust-prune, and backlink
repair. It is not tuple persistence or page writes, and it is not the `~492s`
debug-build artifact that originally gated the landing discussion.

No Task 29d parallel-build blocker should be opened for this landing slice.
Future work should target pass-1 Vamana graph construction with fresh
release-mode packet-local measurements.

## Artifacts

See `artifacts/manifest.md`.
