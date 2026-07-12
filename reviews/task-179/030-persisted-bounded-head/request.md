---
agent: codex
role: coder
model: gpt-5
date: 2026-07-12
seq: 01
---

# Review request — Packet 030 persisted bounded head

This checkpoint removes the physical read path's O(N) owner seed scans and
persists the FR-080 coordinator head object under the exact epoch identity.

## Commit

- `961056f6b` — persist manifest-bound DistANN head samples.

## Behavior

- Build derives a deterministic bounded entry-region sample from the stitched
  graph, computes a domain-separated canonical digest, and binds that digest in
  both build specification and v2 epoch manifest instead of the former zero
  placeholder.
- The epoch object is stored as one coordinator head-state row plus ordered
  row-wise vectors keyed by `(index_oid, logical_index_uuid, build_id)`. Row-wise
  storage avoids imposing one varlena limit on `C × dimensions` and works when a
  future coordinator is outside the physical owner roster.
- Sample construction discovers every directed entry region, reserves one
  deterministic entry per region, fails closed if `head_index_cap` cannot cover
  them, then fills the remaining budget in round-robin BFS order.
- Physical scan open selects the candidate by exact v2 fingerprint, decodes its
  registered manifest/options, validates sample count/dimensions/digest, and
  rebuilds a deterministic in-memory Vamana head with bounded-degree
  reachability repair.
- Queries seed exclusively from this local bounded object. If the requested
  seed budget covers the complete sample, every sample vector is scored; larger
  samples use head-graph search. The temporary remote O(N) seed endpoint and
  transport call are removed.
- Unpublished abort removes the head object atomically with the Aborted gate.
  Published predecessors retain it through retirement and lose it only when
  token-safe reclaim becomes Applied; the active successor remains intact.

## Live evidence

The focused PG18 three-owner fixture validates a nonzero bounded head digest,
deterministic sample count across identical builds, abort cleanup, a 30-row
three-owner CustomScan seeded only from the persisted head, and predecessor-only
head removal during reclaim.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit remaining work

- Replace the legacy pruning-oriented CLI fixture with real physical builds on
  three PostgreSQL instances, including coordinator-outside-roster and
  one-owner cases.
- Run the required 10k/50k/100k `ecaz bench suite` A/B recall, latency, and
  storage matrix. The bounded-head recall sensitivity must be measured rather
  than inferred from this 30-row correctness fixture.

Leaving this request open for outside review.
