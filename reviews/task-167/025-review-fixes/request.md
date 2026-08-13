---
agent: codex
role: coder
model: GPT-5
date: 2026-08-13
seq: 1
---

# Task 167 packet 025: intent-gated frontier retry and saturated target

This checkpoint addresses the findings in packet 024 feedback:

- `OwnedRecordMissing` remains strict for ordinary missing records, while a
  bounded frontier retry is permitted only when the owner has an active
  `ec_distann_remote_prepared_xact_intent` row. The retry waits briefly for the
  prepared transaction to resolve and then re-reads under a fresh snapshot.
- `traversal_frontier_retries` is exposed in the stage counters. The fixture
  resets and records it during concurrent churn and again for a steady query.
- The concurrency drill now makes the shared target full (`before_neighbors=8`)
  and runs a second concurrent insert wave through the full-degree robust-prune
  path. The target remains bounded (`final_neighbors=8`) and both inserts pass.
- The fixture records the pinned-owner probe outcome. In this run it was
  `zero_rows`; the owner-exact probe remained the serving assertion and passed.
- The fixture comment and packet explain that `forward_neighbors_selected=2`
  counts inserted-node forward edges, while `back_edge_check=true` separately
  proves the controlled reverse backlinks.

Exact-head runtime result:

```
release_profile_preflight status=passed ... extension_git_sha=f0bcb06f8e50908a67568ce583d2e877103c3cc8 extension_build_profile=release
physical_concurrent_insert_query ... role=saturated_target ... before_neighbors=Some(8) final_neighbors=Some(8) inserts_ok=true pass=true
physical_concurrent_insert_query ... role=frontier_retry_counter churn_retries=Some(0) steady_retries=Some(0) pass=true
physical_concurrent_insert_query ... forward_neighbors_selected=2 ... back_edge_check=true pass=true
physical_concurrent_insert_query pass=true
```

The installed extension contains code checkpoint `f0bcb06f8`; the fixture-only
runner/comment checkpoint was committed afterward as `8fdfe828a`. The exact
command and artifact provenance are in `artifacts/manifest.md`.

Validation passed with `cargo check --features pg18` and
`cargo check -p ecaz-cli`. The exact-head PG18 fixture passed with two owners,
100 rows, dimension 4, graph degree 8, remote insert probing, and fault drills
skipped.

This packet remains review-open. The mandated 10k/50k/100k A/B recall,
latency, storage, and insert measurements, including the append-when-room A/B,
are still outstanding and are not claimed here.
