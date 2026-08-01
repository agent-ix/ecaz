# Audit: FR-084 / ADR-085 / ADR-086 vs code

Task 214 P0 slice. Auditor: parallel subagent, 2026-08-01, worktree
`.worktrees/task-203` @ `baf81d498`. Specs read in full; implementation
verified against `traversal_replica.rs`, `gateway_copy.rs`, `head_cache.rs`,
`scan.rs`, `generation_read.rs` replica selection, `options.rs` GUC
registrations, and NFR-021 suite enforcement in
`crates/ecaz-cli/src/commands/bench/suite.rs`.

## F-1 — ADR-086 Decision reversed by shipped behavior
- **spec:** `spec/adr/ADR-086-ec-distann-coordinator-traversal-replica.md` — "Task 198 measured decision … The decision is therefore **PROMOTE to Task 199 productionization** … make Ready-replica preference the normal path without benchmark selectors" (status: ACCEPTED)
- **code:** `src/am/ec_distann/generation_read.rs:3340-3358`; `src/am/ec_distann/options.rs:406-413` (`ec_distann.allow_nonconforming_replica`, default off); `crates/ecaz-cli/src/commands/bench/suite.rs:3696,4229`
- **type:** specified-but-changed · **severity:** high
- ADR-086's Decision and its embedded Task-198 promotion record are the exact opposite of shipped behavior. The replica is reachable only through the off-by-default GUC, labeled non-conforming under NFR-021 clause 4, and inadmissible as a decision-bearing benchmark arm under NFR-022 — Task 210 closed the allowlist so an unsharded coordinator-resident relation with non-zero bytes fails the suite. "Ready-replica preference the normal path" never shipped and was affirmatively reversed. **Needs a superseding ADR** recording the NFR-021/NFR-022 demotion (Tasks 203/210).

## F-2 — TRAV-30 gateway copies shipped but listed as Rejected
- **spec:** ADR-086 Rejected alternatives: "Sparse top-layer or bridge replication (`TRAV-28`–`TRAV-30`) — Not selected…"
- **code:** `src/am/ec_distann/gateway_copy.rs:1-30`; `ec_distann_gateway_copy_stats` at gateway_copy.rs:154; `gateway_copy_capacity()` GUC
- **type:** shipped-but-unspecified · **severity:** high
- TRAV-30 is now the shipped, NFR-021-conforming replacement direction — bounded head-landmark routing payloads (neighbor ids + codes, never vectors), with activation counters. No FR specifies the mechanism, its capacity bound, its fill/re-batch semantics (`fill_gateway_rows`), or its NFR-021 accounting; ADR-086 lists the family under Rejected. The superseding ADR from F-1 should record the reversal, and the mechanism needs an owning FR.

## F-3 — FR-084 framing stale (promotion track vs withdrawn/nonconforming)
- **spec:** FR-084 — replica as ordinary optional performance object; AC-7 anticipates "promotion" (status PROPOSED)
- **code:** `options.rs:407-413`; `generation_read.rs:3344`; `gateway_copy.rs:3` ("The **withdrawn** FR-084 traversal replica")
- **type:** specified-but-changed · **severity:** high
- Shipped reality: replica gated behind an explicitly named "nonconforming" opt-in GUC, never decision-bearing, code calls the program "withdrawn". FR-084 mentions neither the GUC, the NFR-021 classification, nor NFR-022 inadmissibility. `spec/functional/index.md:18` carries the same unqualified framing.

## F-4 — Operator-surface privilege model differs
- **spec:** FR-084 Operator Surface — "SECURITY DEFINER search path, revoke PUBLIC execute" for build/retire/reclaim
- **code:** `traversal_replica.rs:1003-1006,1688-1721` (invoker rights, in-function `require_index_owner` at :323); `src/lib.rs:933-941`
- **type:** specified-but-changed · **severity:** medium
- Shipped functions are invoker-rights with an internal owner/superuser check, not SECURITY DEFINER + PUBLIC revoke. Either the spec sentence or the grants should change.

## F-5 — Extra control/recovery surface unspecified
- **spec:** FR-084 lists exactly four functions (build/status/retire/reclaim)
- **code:** `traversal_replica.rs:1382` (`_mark_traversal_replica_stale`), :1617 (`_guard_traversal_replica_mutation`), :1653 (`_control_preflight`), :1667 (`_recover_traversal_replica_invalidation`), :1592 (VACUUM invalidation), :43-132 (per-backend Ready-presence cache + suppression set), `options.rs` (`replica_control_password_file`)
- **type:** shipped-but-unspecified · **severity:** low
- Four additional SQL-visible control/recovery functions, VACUUM-driven invalidation, a password-file GUC, and a backend-local suppression cache exist with no spec coverage. Consistent with FR-084 intent (mutation still fails closed at :1572-1573), but the operator surface as specified is incomplete.

## F-6 — ADR-085 coordinator-head decision superseded
- **spec:** ADR-085 Decision item 5 "**Coordinator head index** (FR-080)"; D3 fixed cap 4096 breadth-first sample
- **code:** `options.rs:380-405` — `head_replica_count`, `shard_head_storage` (default on), `sharded_head_search` (default on)
- **type:** specified-but-changed · **severity:** high
- NFR-021 clause 3 revoked the coordinator-head exemption; Task 210 made membership-only head persistence plus sharded owner-side head search the default. The coordinator-resident head survives only for single-owner rosters. **Needs a superseding/amending ADR**; the C=4096 cap-retention measurement itself remains valid.

## F-7 — ADR-085 single-global-graph core verified accurate (no drift)
- `shard_build.rs:1-40,990` (closure-overlap shards → per-shard Vamana → streaming k-way stitch → one global graph); Task 179 physical hash-shard generations do NOT contradict ADR-085 items 1–4 / D8 — the build still stitches one global graph; physical shards are disjoint hash-placed partitions of its records/row tier. Drift is confined to the head (F-6) and replica narrative (F-1/F-2).

## F-8 — Stale module-header docs in head_cache.rs
- **code:** `head_cache.rs:7-11` — still describes the pre-FR-082 M0 world (aminsert errors, no epochs)
- **type:** specified-but-changed (stale code documentation) · **severity:** low

## Supersession summary
- **ADR-086** (ACCEPTED): Decision + Task-198 promotion reversed (F-1); Rejected TRAV-30 now shipped (F-2). One superseding ADR covering both.
- **ADR-085** (PROPOSED): Decision item 5 / D3 coordinator-head portion superseded by sharded-head default (F-6); single-global-graph core stands (F-7).
- **FR-084**: edit directly to record opt-in/nonconforming/never-decision-bearing posture (F-3, F-4, F-5).
