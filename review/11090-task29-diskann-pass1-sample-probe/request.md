# Task 29 DiskANN Pass-1 Sample Probe

## Summary

This packet extends `ecaz bench diskann-build-probe` with probe-only pass-1
candidate augmentation and in-memory graph recall, then runs two local PG18
real-10k measurements from the same head:

- baseline build replay, no pass-1 sample augmentation
- pass-1 sample augmentation with `32` candidates selected from a fixed global
  `1024`-row sample

Both commands use `ecaz-cli` with explicit `--host`, `--port`, and `--database`
flags against `task29_diskann_baseline`.

## Result

Baseline in-memory Vamana recall is already effectively perfect at scan list
`100`:

- baseline in-memory `recall@10`: `0.9995`
- pass-1 sample in-memory `recall@10`: `1.0000`

The augmentation does not materially change the graph:

- pass-1 candidate pool mean/p95 changes from `101.93/106` to `114.77/128`
- pass-1 selected mean/p95 changes from `8.22/12` to `8.67/13`
- final out-degree mean changes from `24.50` to `24.64`
- final max in-degree changes from `3250` to `3249`

The important conclusion is not that sample augmentation helps. It is a
negative control. The in-memory graph built with the same source vectors and
Vamana algorithm gets `0.9995` recall while the persisted SQL benchmark from
`review/11088...` reports about `0.931` across the scan-list sweep. That moves
the next landing blocker away from build graph connectivity and toward the
persisted scan/scoring/rerank path.

## Recommendation

Do not promote pass-1 sample augmentation.

Next optimization/debug target: compare persisted scan results against the
in-memory graph search for the same query ids and candidate ids. The likely
failure surface is one of:

- persisted scan traversal is not visiting the same candidates as the in-memory
  graph search,
- persisted scan scoring/rerank is using a different distance than the
  source-vector `1 - inner_product` distance,
- candidate materialization or ordering is dropping/reordering good graph hits.

The next packet should add an `ecaz-cli` comparison command that runs a small
query subset through both paths and emits per-query exact IDs, in-memory graph
IDs, and SQL DiskANN IDs.

## Artifacts

See `artifacts/manifest.md`.
