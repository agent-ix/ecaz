# Task 29 Release Latency Refresh

## Request

Review the release-mode Task 29 latency refresh and pgvectorscale comparison.

Current branch head for the code used by the pgvectorscale comparison:
`8064bf51b339c3b26f69354e325c62d99a57d84a`

This packet addresses the reviewer blocker from
`review/11103-task29-landing-readiness-refresh/feedback.md`: packets `11097`
and `11098` used a non-release `cargo pgrx install`, so their DiskANN latency
rows were debug/dev-mode numbers.

## Summary

The release-mode remeasurement strengthens the Task 29 landing story.

- Release-installed PG18 `ec_diskann` on the real-10k corpus measured
  recall@10 `0.9970` at L=200 and `0.9975` at L=800, with NDCG `0.9999`.
- Release latency dropped from the prior debug/dev packet values of
  `58.5 ms` mean at L=200 and `67.7 ms` mean at L=800 to `8.74 ms` and
  `9.57 ms` respectively.
- Tail latency is now also in the expected range: L=200 p50/p95/p99
  `8.64/9.44/11.5 ms`; L=800 p50/p95/p99 `9.50/10.6/11.3 ms`.
- Backend memory HWM stayed small and stable across the sweep:
  `67,104 KiB` at L=64 to `68,688 KiB` at L=800.
- The previous HNSW reference row from packet `11103` remains materially
  slower at lower recall: HNSW recall@10 `0.9700`, mean `35.25 ms`,
  p50/p95/p99 `33.1/39.4/49.1 ms`.

## pgvectorscale

The pgvectorscale head-to-head was worth running.

Setup:

- Installed local pgvector `0.8.2` against PG18 after the first comparison
  attempt exposed a stale PG17-built `vector.so`.
- Installed local pgvectorscale `0.9.0` against PG18 using an isolated
  `/tmp/pgvectorscale-cargo-pgrx-0.16.1` cargo-pgrx binary, leaving the
  global cargo-pgrx `0.17.0` unchanged.
- Added `ecaz compare vectorscale` so the comparison uses the same ecaz corpus
  and truth/latency machinery instead of bare SQL.

Measured pgvectorscale build/reference row:

- Sidecar load: `10,000` rows from `task29c_phase_profile_corpus`.
- Build options: `num_neighbors=32`, `search_list_size=100`,
  `max_alpha=1.2`, `storage_layout=memory_optimized`.
- Build time: `5.82s`.
- Index size: `5,136,384` bytes (`5016 kB`), versus `ec_diskann`
  `4,939,776` bytes (`4824 kB`) on the same corpus.

Matched release query sweep:

| sweep | ec_diskann recall / mean | pgvectorscale recall / mean |
| --- | --- | --- |
| 64 | `0.9965` / `9.19 ms` | `0.9960` / `3.56 ms` |
| 128 | `0.9965` / `8.06 ms` | `0.9990` / `5.84 ms` |
| 200 | `0.9970` / `10.4 ms` | `1.0000` / `8.85 ms` |
| 400 | `0.9970` / `9.86 ms` | `1.0000` / `16.2 ms` |
| 800 | `0.9975` / `10.1 ms` | `1.0000` / `31.2 ms` |

Interpretation: pgvectorscale is faster at L=64/128/200 and reaches exact
top-10 recall by L=200 on this 10k workload. `ec_diskann` is smaller and its
scan time stays flatter at high list sizes, but this comparison confirms the
first post-landing optimization target: reduce scan constant factors at the
quality point around L=128-L=200.

## Recommendation

Land Task 29 / 29a / 29b / 29c after reviewer sign-off.

The release-mode blocker is resolved: the debug-mode latency concern no longer
applies, and the corrected rows put `ec_diskann` comfortably ahead of the local
HNSW reference while keeping recall near exact. The pgvectorscale comparison is
not a landing blocker, but it gives a concrete target for the next tuning lane:
close the L=128/L=200 scan gap without sacrificing the smaller index footprint
or the flat high-L latency behavior.

## Artifacts

See `artifacts/manifest.md`.
