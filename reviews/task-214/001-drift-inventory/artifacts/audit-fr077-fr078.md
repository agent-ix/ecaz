# Audit: FR-077 (sharded build & stitch) + FR-078 (hash placement & handoff) vs code

Task 214 P0 slice. Auditor: parallel subagent, 2026-08-01, worktree
`.worktrees/task-203` @ `baf81d498`.

## Verified conformant (no finding)
- Placement hash v1 (`fmix64(vec_id XOR 0x64697374616e6e70)`), TC-050 golden vectors pinned (`placement.rs:30-48,191-208`).
- All digest domain strings byte-match the spec (15 domains checked).
- Handoff surface signatures match FR-078 (begin/stage/seal/abort, topology, registry, T1/T2 build, status column shape).
- 107-byte owner-stream hash state with `sha2 = "=0.11.0"` pin + compile-time assert; 303-byte Ready receipt; empty-sequence-zero; 8 MiB × roster router bound; byte totals via `pg_table_size`.
- Frozen v1 validity domain (`generation_descriptor.rs:48-58,595-607`).
- Registration digest v1 field order + golden test; lock order; `EC_BUILD_BUSY` conditional locks; subxact promotion/abort callbacks; gate-clearing commit-scheduled release.
- `require_read_committed` on every handoff/lifecycle endpoint (24 sites).
- All FR-078 error codes present; secret provider grammar matches (`node_registry.rs:28-73`).
- Physical capture excludes dead tuples, re-fetches TID under frozen snapshot, `EC_SOURCE_SNAPSHOT` on mismatch (`ambuild.rs:729-1133`); handoff-entry size preflight before graph construction.

## Findings

### F1 — Closure-overlap reuse mode is not "extract-to-shared" (medium, specified-but-changed)
FR-077 mandates lifting the distance-ratio helper from `ec_spire/build/routing_plan.rs` into a shared module. Code: fresh inline ε-band in `shard_build.rs:8-11,632-703`; no shared helper exists and routing_plan.rs has none to lift. Code doc-comment quotes FR-077 wording ("implement the ε band fresh") that is not in the current spec. Behavioral outcome equivalent; the extraction mandate was dropped.

### F2 — Shard/stitch stats and build wall time never reach the epoch manifest (high, specified-but-changed)
FR-077 + AC-3 + CON-4 require shard count, closure duplication factor, stitch edge-union stats, build wall time, peak-memory row in the epoch manifest. Code: `ShardBuildStats` (`shard_build.rs:105-138`) emitted only via `pgrx::log!` (`ambuild.rs:994-1001,1462-1477`); `DistannEpochManifestV2` (`manifest_v2.rs`) has no such fields; wall time measured nowhere. AC-3/CON-4 unsatisfiable as written.

### F3 — Post-stitch reachability-repair pass unspecified (medium, shipped-but-unspecified)
`repair_reachability` (`shard_build.rs:1247-1348`, invoked :627) BFS-checks from the medoid and appends/evicts edges on nearest reached sources (protected-edge bookkeeping) — mutating adjacency beyond the specified union+prune, including single-membership passthrough records, to guarantee CON-3. Mechanism and `reachability_repairs` stat unspecified.

### F4 — Auto shard-count policy defined nowhere (low, shipped-but-unspecified)
`resolve_shard_count` (`shard_build.rs:576-586`): ≤20k nodes → 1 shard; else `(n/25_000).clamp(2,16)`. FR-078 delegates to FR-077, which never specifies it.

### F5 — `ec_distann_recover_epoch_publish` takes a build id; spec requires build-id-free (high, specified-but-changed)
FR-078:266-268 and FR-082:223 declare `(index_regclass)` only. Code: `t4a.rs:9` requires `build_id: Uuid`, validated against the durable decision. An operator following the spec cannot call the function.

### F6 — Build status never emits `last_error_category` (medium, specified-but-changed)
`build_coordinator/mod.rs:1294-1305` hardcodes the ninth column to `None`; no error-category storage exists.

### F7 — Build status fails closed on any remote participant (medium, specified-but-changed)
`build_coordinator/mod.rs:1263-1267` — "remote participant status is not yet implemented" error for multi-node rosters; spec requires aggregation by build id with no local-only restriction.

### F8 — `ec_distann_build_epoch_with_training` endpoint + training-relation contract unspecified (medium, shipped-but-unspecified)
`t2.rs:14-24,26-93`: fourth arg `training_relation regclass` with required shape (`training_ordinal bigint` + `vector real[]`, ordinal-ordered, `EC_HEAD_TRAINING` errors); absent from FR-078's operator surface. Trained-vs-untrained replay conflict (`EC_BUILD_ID_CONFLICT`, t2.rs:250-281) likewise unspecified.

### F9 — `head_sample_digest` is a membership digest under the shipped default (high, specified-but-changed)
FR-078 build-spec table says "canonical coordinator head sample". Code: with `shard_head_storage` default true and multi-owner roster, `t2.rs:383-405,751-766` binds `membership_digest()` into the immutable build spec/manifest and persists an empty coordinator head graph. Neither the GUC nor the membership-digest form appears in any spec (Task 210 outran FR-078; cross-ref FR-080 audit F3).

### F10 — Unspecified GUC roster lane carrying raw conninfo (medium, shipped-but-unspecified)
`roster.rs:36-70` — `ec_distann.roster` (`node_id@conninfo`, raw libpq conninfo in a userset GUC), `ec_distann.local_node_id`, `ec_distann.epoch`; consumed by `placement_directory_for_epoch` on the scan path. FR-078 explicitly excludes GUC-supplied identity; the M2 lane still ships and drives multi-node scans, unbounded by any spec.

### F11 — Begin-build rejects partitioned/inheritance sources (low, shipped-but-unspecified)
`t1.rs:283-299` — `pg_inherits` probe, format-v1 constraint in no FR text.

### F12 — Preload prerequisite + global gate serialization lock unspecified (low, shipped-but-unspecified)
`t1.rs:253-254` — `require_shared_preload()` and `lock_global_gate_serialization(false)` precede registration; operationally important preconditions absent from spec.

### F13 — Trained-head option requires `head_index_cap == 4096` exactly (low, shipped-but-unspecified)
`generation_descriptor.rs:63,619-628` — a fourth v2 validity requirement rejecting otherwise spec-valid option bytes.

### F14 — "Manifest digest" wording vs candidate digest returned (low, specified-but-changed)
T2 returns `candidate.digest()` (`t2.rs:287,700,791`) fresh and on replay; FR-078's "return the existing 32-byte manifest digest" is wrong as written; should name the candidate digest.

## Slice behaviors in NO distann spec (grep-confirmed)
- **Stage/latency attribution subsystem** — `stage_counters.rs` (37 query stages, 32 materialization-work counters, speculative attribution committed on traversal success). Benchmark-critical (`traversal_frontier_insert` proves pushdown); no spec names any stage or counter.
- `ec_distann_fold_delta_into_graph` (`insert.rs:483`) — absent from FR-083's surface.
- `ec_distann_prepare_control_rebuild` (`generation_store.rs:1059`).
- `ec_distann_reclaim_cancelled_generation` (`participant_lifecycle.rs:869`) — only in `spec/reviews/*` narrative, no normative FR.
- Gateway-copy stats surface (`gateway_copy.rs:155`).
- `shard_head_storage` GUC + membership-only persistence (F9).
- `ec_distann.roster`/`local_node_id`/`epoch` GUC lane (F10).
- `ec_distann_test_set_conninfo_secret` (test-only; completeness).
