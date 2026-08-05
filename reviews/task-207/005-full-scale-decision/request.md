---
agent: claude
role: coder
model: gpt-5
date: 2026-08-04
seq: 1
---

# Task 207 full-scale decision

> Superseded by `../006-re-review-corrections/` for effective seed wording,
> physical activation attestation, owner-lane interpretation, and search-path
> disposition.

This packet closes the corrected construction/search A/B requested by review.
Both arms use fixed `build_shards=4`; only the head construction toggles:
`stitched_bfs` control versus `partition_union` candidate. The candidate uses
the full per-shard cap and carries the persisted construction marker. The
production search path remains `persisted_head`; the release build's effective
seed derivation at BW128 was 256/256 because benchmark-only seed GUCs were
compiled out. No production default is changed.

## Decision

The construction hypothesis is not a universal win and is not promoted. The
union candidate is faster at 10k and 50k but slower at 100k; recall is lower at
10k and 50k and slightly higher at 100k. Construction is also marginally
slower at 50k and 100k, while storage is effectively identical. Keep
`stitched_bfs` as the default and retain `partition_union` as an experimental
explicit arm. The 100k sign behavior is recorded as the finding, not averaged
away.

The old owner-oracle lane was run separately with `owner_scan`, but its output
was head-independent and captured top-k 32 rather than the main top-k 200. It
is withdrawn from membership/overlap evidence and is not substituted for the
release A/B latency decision.

NFR-021 was preregistered as conforming for every physical arm. All three
scales have recall, latency, and storage evidence with 50 timed queries and 10
warmups. The corrected suite audit passed all six release A/B steps.

## Evidence

See `artifacts/result-summary.md` and `artifacts/manifest.md`. The release
matrix and owner-control raw summaries remain under the preregistered packet
at `../004-search-and-sharding/artifacts/` and are cited by exact path there.
