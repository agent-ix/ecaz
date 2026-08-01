# Audit: FR-080 coordinator head index vs code

Task 214 P0 slice. Auditor: parallel subagent, 2026-08-01, worktree
`.worktrees/task-203` @ `baf81d498`. Confirmed the expectation: FR-080 is the
most drifted spec in the set — its core premise (a coordinator-resident head
answering first hops with zero network round trips) is the exact opposite of
the shipped default (sharded, membership-only head served by owners over RPC).

## F1 — Coordinator-resident head no longer the shipped model
- **spec:** FR-080 Description — "The coordinator SHALL maintain an in-memory head index … so a query's first hops execute locally with zero network round trips"
- **code:** `options.rs:44,59` (both sharded-head GUCs default true, registered :390-405); sharded read path `generation_read.rs:3671-3699`
- **type:** specified-but-changed · **severity:** high
- Default shards the head across the roster: coordinator keeps only a bounded membership list and fans `head_search` to owners, who read landmark vectors locally and return ≤ seed_count seeds each (`generation_read.rs:3540-3657`). `generation_read.rs:3674-3676` makes the persisted shape, not the GUC, authoritative — a membership-only head cannot take the coordinator-local path.

## F2 — AC-1 contradicted by default behavior
- **spec:** FR-080-AC-1 — "Head-index search returns entry candidates without any remote call"
- **code:** `generation_read.rs:3638-3651` (per-shard RPCs); owner endpoint `ec_distann_head_search_physical` at :2229
- **type:** specified-but-changed · **severity:** high
- AC-1 passes only under `--local-head` (legacy) or single-owner rosters. Needs inversion: "no landmark vector crosses the wire; coordinator merges ≤ seed_count seeds per owner" (`head_sample.rs:1282-1300` `merge_head_seeds`).

## F3 — Head persistence is a membership blob, not an epoch-versioned vector sample
- **spec:** Behavior bullet 1 — sample "persisted with the epoch as an epoch-versioned object in the index relation"
- **code:** `head_sample.rs:755-785` (u32 LE count + u64 LE vec_ids), :837-851 (blob on `ec_distann_generation_head_state`, zero `_head_sample` rows); `build_coordinator/t2.rs:382-405` (membership digest + deliberately empty persisted head graph); t2.rs:765 (membership-only ⇔ `shard_head_storage() && roster.len() > 1`)
- **type:** specified-but-changed · **severity:** high
- Coordinator head persistence is one bounded blob (4 + 8·C bytes); single-owner rosters keep the full-vector shape — a distinction the spec lacks. Fixture asserts 0-byte coordinator head relations as NFR-021 control (`distann_multicluster.rs:5404,5521`). Blob format, state-row schema, membership digest, zero-row invariant: all unspecified.

## F4 — Sample is BFS over stitched graph, not per-build-shard union
- **spec:** Behavior bullet 1 — "breadth-first traversal from each build shard's entry medoid over that shard's graph, bounded by hop radius, per-shard samples unioned"
- **code:** `head_sample.rs:239-329` (`build_head_sample`: BFS regions over the final stitched graph, global medoid + component seeds, round-robin depth fill; no per-shard medoid, no hop radius)
- **type:** specified-but-changed · **severity:** high
- This is the Task 203 "head from stitched graph not per-partition union" finding; FR-080-AC-3's reachability property is tested against components of the stitched graph, a weaker property.

## F5 — Serving-side head graph is per-shard, owner-built
- **spec:** Behavior bullets 3-4 — coordinator constructs in-memory head from persisted sample on first use; legacy generations search the persisted Vamana head graph
- **code:** `generation_read.rs:1250-1394` (owner materializes its shard from locally held vectors, builds fresh per-shard Vamana, cached in `OWNER_HEAD_SHARD_CACHE` cap 4 at :482); `head_sample.rs:1116-1243` (`build_owner_head_shard`, per-shard seed `seed ^ ordinal·0x9e3779b97f4a7c15`)
- **type:** specified-but-changed · **severity:** high
- In the default path no node ever searches the persisted head graph (empty, F3); each owner builds a navigable graph over its shard only (comment cites DISTRIBUTEDANN §3). Even legacy full-vector generations load a build-time-persisted graph rather than rebuilding; only the pre-generation M0 path (`head_cache.rs:108-149`) matches the spec wording.

## F6 — §4.1 head replicas with population attestation: entirely unspecified
- **code:** `options.rs:381-389` (`head_replica_count`); `generation_read.rs:2044-2185` (`ec_distann_populate_head_replicas`: export→import per (shard, replica) pair, attestation row in `ec_distann_head_replica_state` only after ALL pairs placed :2150-2181); routing gate :3502-3532 (attested ≥ session GUC) with clamp-to-owner fallback :3582-3606; query-digest server selection `head_sample.rs:1245-1280`; replica serving keyed by members-derived ordinal `generation_read.rs:1261-1279,1336-1362`
- **type:** shipped-but-unspecified · **severity:** high
- A complete DISTRIBUTEDANN §4.1 subsystem — export/import endpoints (:1966-2217), copy table, per-epoch attestation, requested-vs-attested routing check, load spreading — exists with no owning requirement. The attestation semantics are load-bearing correctness rules living only in code comments and Task 210 packets.

## F7 — Members-derived shard identity unspecified
- **code:** `placement.rs:78-103` (`shard_owner_ordinal`, mixed ownership rejected); consumed for cache key and graph seed `generation_read.rs:1268-1293`
- **type:** shipped-but-unspecified · **severity:** medium
- A replica serving a foreign shard must key/seed with the owner's members-derived ordinal or the identical shard gets different topology per serving node (Task 210 packet 005 finding 2). Determinism invariant essential to replica/owner equivalence; in no spec.

## F8 — Cache clause matches one of four caches; legacy cache violates it
- **spec:** Behavior bullet 3 — 4-tuple key, at most two immutable epoch entries, LRU, Userset off switch
- **code:** conforming: `generation_read.rs:262,351-390`, `options.rs:70,443-446`. Not covered: `RETAINED_EPOCH_CACHE` cap 4 (:392), `OWNER_HEAD_SHARD_CACHE` cap 4 (:482), prepared-query cache cap 4 (:422) — none honor the off switch; legacy `head_cache.rs:75-105` is process-wide unbounded per-oid HashMap, no LRU/off switch, still consumed by `remote_endpoint.rs:404`, `dml.rs:193`, `remote_transport.rs:2377`, `routine.rs:366`
- **type:** specified-but-changed (partially conformant) · **severity:** medium
- Relcache invalidation of these caches (`generation_read.rs:286-345`) also unspecified.

## F9 — Trained head scoring distributed per-owner, not coordinator-exact
- **spec:** Behavior bullet 4 — trained generations "exact-score at most C persisted vectors … at most 32 seeds"
- **code:** policy plumbed through the sharded RPC (`generation_read.rs:3690-3696`, owner-side :1259, exact-scores its shard `head_sample.rs:1350`); constants conform (`generation_descriptor.rs:62` = 200, `head_sample.rs:20` = 32)
- **type:** specified-but-changed · **severity:** medium
- No coordinator-persisted vectors to exact-score under the default; each owner scores its shard's landmarks, coordinator merges/dedups/truncates. Execution locus in the spec is wrong for multi-owner rosters.

## F10 — CON-1 memory bound names the wrong holder
- **spec:** FR-080-CON-1 — "Head-index memory SHALL be bounded by C × (vector bytes + graph overhead)"
- **code:** coordinator holds 4 + 8·C bytes + ≤ seed_count merged seeds; each owner ~C/roster vectors + per-shard graph; replicas additionally hold imported foreign shards (`generation_read.rs:1364-1443`)
- **type:** specified-but-changed · **severity:** medium

## F11 — Legacy local head demoted to nonconforming fixture control, unrecorded
- **code:** `distann_multicluster.rs:177-182` (`--local-head`, conflicts_with sharded_head, "Control arm … now that the sharded head is the shipped default (fe5822f46)"), applied :1180-1192, 5982-5989, arm GUC :1384-1400; suite plumbing `suite.rs:700,5341-5342`
- **type:** specified-but-changed · **severity:** medium
- A reader implementing from FR-080 today would build the control arm. Rewrite around the sharded default; mark coordinator-resident head as the single-owner/legacy degenerate case.

## Head behaviors in NO distann spec (grep-confirmed)
1. GUC surface: `shard_head_storage`, `sharded_head_search`, `head_replica_count` + "persisted shape overrides the GUC" rule (`generation_read.rs:3674-3686`).
2. Membership blob wire format + zero-row sample table + membership digest domain (`head_sample.rs:755-785,202-216`).
3. Replica population attestation protocol (endpoints, all-pairs-then-attest, attested≥requested gate, clamp-to-owner fallback).
4. Query-digest head-shard server selection (`head_shard_server`, `head_sample.rs:1245-1280`).
5. Members-derived shard ordinal (`placement.rs:78-103`) as cache-key + graph-seed identity.
6. `head_replica_shards_served` counter (`stage_counters.rs:215,287`) + `head_replica_fallbacks` (`generation_read.rs:3600-3604`) — both `distann-head-attribution-benchmark`-gated, so replica serving is unprovable in production builds — worth a spec decision.
7. Per-owner seed merge contract (`merge_head_seeds`: deterministic (dist, vec_id) order, dedup, truncate; `head_sample.rs:1282-1300`).
8. Owner head-shard per-backend cache (cap 4, keyed on digest of ordinal+members+build params+policy, `generation_read.rs:476-560`; rebuild cost ~7 s/query at 100k per comment).
