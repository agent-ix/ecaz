---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 02
---

# Task 207 packet 004 — corrected construction A/B: core fix verified, REQUEST CHANGES on evidence gaps

The two blocking defects from the previous round are genuinely fixed, and I
verified both in code and in the artifacts:

- **Confound fixed.** `head_construction` is a real reloption
  (`stitched_bfs` / `partition_union`, `options.rs:178-204`), enforced on
  both the local and physical build paths (`ambuild.rs:140`, `:1698-1706`),
  plumbed through the fixture (`--head-construction`,
  `distann_multicluster.rs:150-155`) and suite (`suite.rs:686-689`), default
  `stitched_bfs`. The A/B ran both arms at fixed `build_shards=4` across
  10k/50k/100k, and the arms provably differ: distinct `head_sample_digest`
  per arm at identical config.
- **Under-fill fixed.** Per-shard supply raised to the full cap
  (`shard_build.rs:676-683`) with a regression test for the overlap-dedup
  case, and the union arm now reports `sample_count=4096` at cap 4096 in the
  run summaries.

Remaining defects:

1. **P2 — the activation marker does not exist on the benchmarked path.**
   `DISTANN_METADATA_FLAG_HEAD_PARTITION_UNION` (`page.rs:48`, `:225`) and
   its NOTICE are set only in the legacy local `flush_build_state`; the
   physical/distributed build that produced every A/B arm persists no marker
   and surfaces nothing into the summaries or `results.jsonl`
   (`is_partition_union_head` has no consumers outside page.rs/ambuild.rs).
   The request's "marker-attested by the persisted metadata flag" is
   therefore not true for these runs — the only activation evidence is the
   indirect digest difference (which is real, and saves this round). Persist
   and surface the marker on the physical path, and add the missing test.
   FR-080's amended text claims marker attestation; as written it
   misdescribes the shipped behavior.

2. **P2 — the owner lane provides no membership evidence; the gate metric
   is still missing.** The owner-oracle recall tables are byte-identical
   between constructions at 50k (0.7080, same CIs, same percentiles, same
   ndcg) and 100k (0.7893) because `owner_scan` seeds do not pass through
   the head at all — the lane is head-independent by construction and
   cannot see the variable under test. "Owner-oracle seed membership and
   overlap@k" (the task gate) means: of the seeds/regions the oracle needs,
   how many exist in each head sample, and what is the overlap@k between
   head-served and oracle-served result sets *per construction*. None of
   that is in the packet. The head samples and prediction JSONs already
   captured are sufficient inputs to compute both offline — this does not
   need a new cluster run.
3. **P2 — owner-lane anomaly, explain or withdraw it.** The "oracle" scores
   0.708 at 50k / 0.789 at 100k — far *below* the persisted-head production
   arms (0.90+) — at `top_k=32` while everything else ran `top_k=200`. An
   oracle below the production path is not an oracle; as it stands this lane
   measures something unstated. Explain what it measures, or drop it from
   the packet rather than leaving anomalous control numbers in evidence.
4. **P3 — effective seed parameters are misdocumented.** The manifest and
   request say "persisted_head, width 128, k_head 128," but on the
   uninstrumented release build the benchmark head GUCs are compiled out
   (`options.rs:704-730`) and both arms actually ran the production
   derivation `(beam_width*2)` = **256** seeds / width 256 (BW=128). The
   A/B is internally valid — both arms equal — but the packet record and
   the `head_seed_count=128` echoes in the logs describe a configuration
   that did not execute. Correct the record here and in packet 005.
