# Task 203 / 001 — artifacts manifest

- Task bucket: `reviews/task-203/`
- Packet: `001-decision-reaudit`
- Branch: `task-203-ec-distann-conformance` (off `origin/main` @ `78b46889c`,
  plus cherry-picked `c830b184f` for the ec_distann 201/202 task docs)
- Head SHA at audit: `b15f3ffb1`
- Date: 2026-07-29
- Host: Intel desktop (local bench host)
- Lane / fixture / storage format / rerank mode: **not applicable — no benchmark
  was run.** This packet is a documentary audit; every finding is a re-read of
  evidence already committed to the repository.
- Isolated vs shared surfaces: not applicable, no measurement taken.

## Nature of this packet

There are no new measurement artifacts. `NFR-007` provenance is satisfied by
citing pre-existing artifacts in their owning packets rather than by copying
them here; each citation below is `path:line` and resolvable at the SHA above.
No number in `request.md` was produced by this audit — all are quoted from
existing packets, task files, spec files, or source.

## Reference paper

`DISTRIBUTEDANN: Efficient Scaling of a Single DISKANN Graph Across Thousands of
Computers`, arXiv:2509.06046v1, 7 Sep 2025. Local copy:
DistributedANN, arXiv:2509.06046 (8 pages). Sections cited:
§2.2 (index layout modifications, head index), §2.3 (near-data computation,
Algorithm 1), §2.4 (orchestration service, Algorithm 2), §3 (graph
construction), §4 (evaluation parameters), §4.1 (scaling), §4.2 (reliability).

Production parameters quoted from §4: `H=5, BW=128, R=72, k=L=200, k_head=200`,
head index 2.5 billion vectors over a 50 billion vector slice. Figure 4 grid:
"H from 4 to 8, BW = 32i for i from 3 to 6".

## Citation index

### Defect 1 — traversal regime

| Claim | Source |
| --- | --- |
| "wide beam, few rounds is the only viable multinode shape ... multinode wants >=32" | `reviews/task-162/003-g0-killcheck/request.md:10-26` |
| BW32/H8 0.9940 @ 12.3 ms, projected 20.3--28.3 ms; BW4/H64 projected 77.6--141.6 ms | `reviews/task-162/003-g0-killcheck/artifacts/manifest.md:63-68` |
| Kill-check grid: BW {4,32} x H {1,2,4,8,16,32,64}, 50k | `reviews/task-162/003-g0-killcheck/task-162-killcheck-suite.json` |
| current default BW=4 | `src/am/ec_distann/mod.rs:253` |
| current default H=100 | `src/am/ec_distann/mod.rs:260` |
| `ECDISTANN_MAX_BEAM_WIDTH = 64` | `src/am/ec_distann/mod.rs:254` |
| "provisional until the M0 recall-vs-H kill-check measurement pins it" | `src/am/ec_distann/options.rs:331` |
| BW=4 provenance: Task 168 local SIMD batching A/B | `src/am/ec_diskann/options.rs:29-32`; `reviews/task-168/002-batched-beam-ab/request.md:20` |
| BW/H are not sweep axes; `top_k` is the quality knob | `crates/ecaz-cli/src/profiles.rs:218-235` |
| seed count `max(BW*2, 32)` | `src/am/ec_distann/generation_read.rs:2650` |
| fixed-product BW16/H25 result | `reviews/task-179/066-complete-finding-benchmarks/comparison.md:28-38` |
| reviewer BW=4 per-RPC finding | `reviews/task-179/060-recovery-state-closeout/feedback/2026-07-13-01-reviewer.md:112-117` |

### Defect 2 — pushdown

| Claim | Source |
| --- | --- |
| threshold hardcoded `None` at the only orchestration call site | `src/am/ec_distann/scan.rs:215` |
| production physical expander discards `_code_threshold` | `src/am/ec_distann/generation_read.rs:3146-3149` |
| replica expander discards `_code_threshold` | `src/am/ec_distann/traversal_replica.rs:2455-2458` |
| only the legacy expander honors it | `src/am/ec_distann/expand.rs:127-137` |
| wire SQL and bound params (physical) | `src/am/ec_distann/remote_transport.rs:567`, `:938-946` |
| `code_threshold` defaults NULL, outside correctness guarantees | `spec/functional/distann/read/FR-079-distann-remote-expansion-protocol.md:45-47`, `:115-123` |
| FND-006 resolution | `spec/reviews/failure-domain.md:40` |
| exact distance owner-side | `FR-079:97-106`; `generation_read.rs:3103`, `:3139`, `:3192-3194` |
| owner-side scoring present (conformant) | `generation_read.rs:3182-3190` |
| Task 194 packet 007 candidate signature | `reviews/task-194/007-fixed-work-candidate/request.md:36-46` |
| Task 194 suite config (BW8/H50) | `reviews/task-194/007-fixed-work-candidate/artifacts/task194-fixed-work-100k.json:33-34` |

### Defect 3 — head index

| Claim | Source |
| --- | --- |
| exact scoring cannot select absent entry nodes | `plan/tasks/181-ec-distann-head-coverage-landmarks.md:27-30` and table `:18-25` |
| three heads, same top-32 seeds, same 0.9625 | `plan/tasks/185-ec-distann-gateway-landmark-selection.md:20-23` |
| `training_landmarks` defined as diagnostic | `plan/tasks/181-...md:108-110` |
| training-query frequency ranking | `src/am/ec_distann/head_sample.rs:452-497`, `:516-537` |
| BFS-prefix / component round-robin sample | `src/am/ec_distann/head_sample.rs:213-303`, `:236-289` |
| monolithic BFS-from-medoid path | `src/am/ec_distann/ambuild.rs:1645-1682` |
| built from stitched global graph | `src/am/ec_distann/shard_build.rs:587-589`; `ambuild.rs:122-169` |
| default `build_shards = 1` | `src/am/ec_distann/mod.rs:247` |
| exact-scan search path (4,096 IPs + full sort) | `src/am/ec_distann/head_sample.rs:1048-1050`, `:1130-1165` |
| head coordinator-local only | `src/am/ec_distann/generation_read.rs:2318` |
| thread-local 2-entry epoch cache | `src/am/ec_distann/generation_read.rs:261-277` |
| `TRAINED_HEAD_SEED_COUNT = 32` | `src/am/ec_distann/head_sample.rs:20` |
| FR-080 claims per-shard union (unimplemented) | `spec/functional/distann/read/FR-080-distann-coordinator-head-index.md:22-27` |
| FR-080 claims 2-entry LRU with 4-tuple key | `FR-080:44-52` vs `src/am/ec_distann/head_cache.rs:75-106` |
| `HEAD-11` unmeasured, `HEAD-12` deferred | `plan/design/ec-distann-recall-latency-roadmap.md:226-227` |

### Defect 4 — replica

| Claim | Source |
| --- | --- |
| replica holds full-precision vector per vec_id | `src/am/ec_distann/traversal_replica.rs:275-283`; `spec/functional/distann/FR-084-...md:26-28` |
| replica columns | `src/am/ec_distann/traversal_replica.rs:448-464` |
| NFR-017 excludes replicated full index from satisfying the gate | `spec/non-functional/NFR-017-distann-latency-recall-gate.md:38-39` |
| NFR-018 excludes the lane; `non-owner graph records = 0` | `spec/non-functional/NFR-018-...md:36`, `:62`, verification duty `:66` — **all at `78b46889c`, before this packet's amendment renumbered the file** |
| FR-078 coordinator stores only its own shard | `spec/functional/distann/build/FR-078-distann-hash-placement.md:492-501` |
| ADR-086 cites no NFR; acknowledges linear per-coordinator amplification | `spec/adr/ADR-086-ec-distann-coordinator-traversal-replica.md:163-166` |
| ADR-086 per-coordinator storage ceiling | `ADR-086:79-86`; measured result `:144-147` |
| ADR-067 storage-scale-out rejection rationale | `spec/adr/ADR-067-spire-customscan-distributed-scan.md:47-51`, `:198` |
| Task 190 narrowing dropped `TRAV-30` | `plan/tasks/190-ec-distann-architecture-escalation-gate.md:70-79`, `:100-112` |
| Task 190 storage budget 2,496,626,688 bytes/coordinator | `plan/tasks/190-...md:113-118` |
| Task 198 Phase 2 linear per-coordinator amplification | `plan/tasks/198-ec-distann-coordinator-traversal-replica.md` Phase 2 capacity bullet |
| Task 201 frozen control contains the replica | `plan/tasks/201-ec-distann-post-replica-latency-residual.md:34`, `:43-44`, `:113` |

### Defect 4b — storage evidence

| Claim | Source |
| --- | --- |
| scalars computed before the variant loop | `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:5153-5160` |
| reprinted unchanged inside the variant loop | `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:5209-5212` |
| identical storage rows across arms, 100k | `reviews/task-198/004-isolated-100k/artifacts/run/replica-isolated-ab-100k/distann-multinode-summary.log:163,166` |
| same in results.jsonl | `reviews/task-198/004-isolated-100k/artifacts/run/results.jsonl:158,161`; `reviews/task-199/003-release-matrix-and-decision/artifacts/run/results.jsonl:53,56` |
| replica `relation_bytes=1659518976`, log-only | `reviews/task-198/004-isolated-100k/artifacts/run/replica-isolated-ab-100k/distann-multinode-summary.log:13` |
| NFR-018 ratio emitter exists | `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:7419-7482`, ratio at `:7470-7475` |
| emitter ran for Tasks 172 and 197, not 198/199 | `reviews/task-172/001-real-multinode-benchmark/`, `reviews/task-197/001-multinode-release-preflight/` |
| conflicting ratios 66.5% vs 52.0% | `reviews/task-198/005-full-scale-decision/artifacts/manifest.md:67-71`; `reviews/task-199/003-.../artifacts/manifest.md:19-23` |
| reviewer objection, overridden | `reviews/task-199/003-release-matrix-and-decision/feedback/2026-07-25-01-reviewer.md:195-199`, `:201-203` |

## Provenance commands

Run from the repository root at the SHA above. Read-only.

```
# Provenance of the NFR-018 full-replica exclusion clause
git log -S "not a valid NFR-018 distributed measurement lane" --oneline \
  -- spec/non-functional/NFR-018-distann-space-amplification.md
#   -> 32b9b43fb
git show --stat --format="%H%n%ad%n%s" 32b9b43fb
#   -> Fri Jul 10 09:48:58 2026 -0700, "spec(distann): define physical
#      hash-shard generations"; touches plan/tasks/172-...md in the same commit

# NFR-018 terms absent from the replica packets
grep -rn "NFR-018\|space amplification\|non-owner graph\|storage budget" \
  reviews/task-198/004-isolated-100k reviews/task-198/005-full-scale-decision \
  reviews/task-199/003-release-matrix-and-decision
#   -> zero matches

# Replica bytes never reach results.jsonl
grep -c relation_bytes reviews/task-198/004-isolated-100k/artifacts/run/results.jsonl
#   -> 0

# Threshold pushdown is inert
grep -rn "expand_nodes(" src/am/ec_distann/ | grep -v "fn expand_nodes"
#   -> the sole orchestration site passes None (scan.rs:215)
grep -n "_code_threshold" src/am/ec_distann/generation_read.rs \
  src/am/ec_distann/traversal_replica.rs

# BW/H pinning across suites
grep -rl "beam_width" reviews/task-*/**/*.json
```

## Task-file inventory audited

`plan/tasks/`: 161, 162, 163, 164, 165, 166, 167, 172, 179, 180, 181, 182, 183,
184, 185, 186, 187, 188, 189, 190, 191, 192, 193, 194, 195, 196, 197, 198, 199,
200, 201 (ec_distann lane), 202. Statuses read at the SHA above.

## Numbering hazard

`origin/main` carries `plan/tasks/201-task38-interrupt-poll-followups.md`; the
ec_distann lane carries `plan/tasks/201-ec-distann-post-replica-latency-residual.md`
(commit `c830b184f`, unmerged to main at audit time). **Task 201 is
double-allocated.** Cite the ec_distann 201 by explicit branch and path, matching
the convention `StR-008` already uses for the 141--146 collisions. Verified by
`git ls-tree --name-only origin/main:plan/tasks`.

## Outstanding for packet 002

1. Full arm-blind storage sweep across all distann packets (the `T4 = ?` rows).
2. NFR-018 ratio-row presence check on every gate packet.
3. PENDING rows: 161, 163, 167, 183, 189.
