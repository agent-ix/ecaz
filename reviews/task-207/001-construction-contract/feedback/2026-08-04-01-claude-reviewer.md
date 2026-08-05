---
agent: claude
role: reviewer
model: claude-fable-5
date: 2026-08-04
seq: 01
---

# Task 207 packet 001 — construction contract: REQUEST CHANGES

Commit `3fb1319af` reviewed in depth (code review performed against the
diff and FR-080/NFR-019/NFR-022 on this branch). The union implementation
itself is genuine and deterministic: `shard_head_nodes` BFS-walks each
shard-local Vamana adjacency from the shard-local medoid *before* stitching,
in global id space, with a hard error if adjacency escapes the partition
(`shard_build.rs:445-478`), and the round-robin dedup/cap union is
deterministic with correct unit tests. This is not a cosmetic re-sampling of
the stitched graph. The plumbing claims (ceiling, forwarding, release-build
NFR-019 tripwire) also verify. But the checkpoint has one structural defect
that invalidates the A/B this contract pre-registers, plus several
overstatements versus FR-080.

## Blocking

1. **P1 — the pre-registered A/B is confounded by construction; the
   isolating arm does not exist.** `build_shards>=2` selects the FR-077
   path — spherical k-means partitioning, per-shard Vamana, stitch-prune,
   `repair_reachability` (`ambuild.rs:998-1041`) — which is a *different
   data-plane graph* than the `build_shards=1` monolithic Vamana,
   independent of any head change. The commit additionally makes the union
   head *unconditional* for shard builds (`head_partition_nodes` is `Some`
   whenever `shard_count >= 2`, and `stage_head_sample_chain` /
   `get_head_sample` always prefer it), so the combination "sharded stitched
   graph + stitched-graph head" — the control that would isolate head
   construction — is unreachable. A `build_shards=1` vs `4` comparison
   measures graph-topology change + head-construction change jointly, with
   no attribution. This violates the task's own gate ("construction is the
   only variable") and the repo A/B-per-change rule. Fix: add a reloption or
   GUC selecting head construction (stitched-BFS vs partition-union)
   independently of `build_shards`, and run the A/B at fixed
   `build_shards=4` toggling only the head. The already-collected 1-vs-4
   runs remain useful as a *combined-effect* observation but cannot close
   phase 1.

2. **P1 — per NFR-022's activation-evidence clause, the union path ships
   with no activation evidence.** No counter, no persisted marker, nothing
   in the generation metadata distinguishes which construction produced the
   head sample. After the Task 205 "pushdown was inert" episode, a green
   candidate run must be able to *prove* the union was active. Emit an
   activation marker (e.g. construction tag in the head-sample chain header
   or a build-time counter surfaced in the multinode summary).

## Should fix

3. **P2 — head under-fill: per-shard prefix cap is exactly `ceil(C/S)` with
   no top-up** (`shard_build.rs:677`). Supply ≈ C, so every duplicate from
   FR-077's closure-overlap band shrinks the final head below cap; the union
   loop exits on cursor exhaustion with `selected.len() < C`. Packet 002
   confirms this empirically: candidate head 3,729 vs 4,096. The candidate
   arm runs at ~91% of the pre-registered cap — a second uncontrolled
   variable, in the direction that *hurts* the candidate. Bound per-shard
   BFS by supply (cap C per shard) or add a top-up pass from unexhausted
   shards.

4. **P2 — the cache claim overstates what shipped.** `DistannCacheKey::legacy`
   is the *only* constructor: `build_id` is `active_epoch` LE-padded and
   `epoch_fingerprint` is the u64 `content_digest` padded
   (`head_cache.rs:82-99`); the real catalog `build_id`/`epoch_fingerprint`
   (available in `build_coordinator/t2.rs`/`t3.rs`/`t4a.rs`) are never
   consulted. FR-080's mandated `ec_distann.physical_epoch_cache` off-switch
   GUC does not exist. Practical aliasing risk is low because every hit
   re-validates `DistannCacheFingerprint` (chain heads, node_count, seed),
   and eviction/promotion logic is correct — but the request's "logical
   index, build, and epoch-derived identity" description is not what the
   code does. Either implement the FR-080 key or say precisely what is keyed.

5. **P2 — the 2-entry bound is global across all indexes and applied to
   owner/DML paths.** The capacity is 2 *total per backend*
   (`head_cache.rs:126-133`), and the same cache serves DML, owner remote
   endpoints, and transport. FR-080's two-entry clause governs the
   coordinator epoch cache; owner-side caching has a separate bounded-keyed
   clause. A backend serving 3+ ec_distann indexes now thrashes a full
   directory+head+codebook rebuild on nearly every alternation. Make the
   bound per-index (two epochs of one index), or split coordinator vs owner
   caches per the FR.

6. **P2 — FR-080 was not reconciled; it was contradicted silently.** The FR
   marks per-partition union as "Successor direction (tracked, not shipped)
   … a change here re-opens this clause," and no spec file is in the diff.
   The task's Spec-reconciliation section makes updating FR-080 in-scope and
   mandatory. Also note the monolithic path's FR-080-AC-3 guarantee (one
   seed per directed entry region, error if regions > cap) has no analogue
   in `build_partition_union_head_sample` — component coverage now rests
   implicitly on `repair_reachability`, untested.

## Notes

7. **P3 — behavior change on monolithic local flush:** `stage_head_sample_chain`
   now routes through `build_head_sample`, which errors when directed entry
   regions exceed the cap; a disconnected local graph that previously built
   now fails. Possibly spec-aligned, but unflagged. Also O(N·dim) transient
   clone of source vectors at flush.
8. **P3 — no tests for the LRU** (eviction, epoch-bump miss, cross-index
   aliasing), none for union component coverage, none for the under-fill.
