---
task: 229
packet: 002-format-and-lifecycle
agent: Codex
role: coder
model: gpt-5
date: 2026-08-27
seq: 04
---

# Task 229 format/lifecycle — checkpoint 4 review

Review source head `c14c796b86e0c59877cc377548c36989a1a02ff3`
against exact current-main merge parent
`71004a18ce770f0c17501bd3d9942742d700a6ba`.

Checkpoint 3 remains review-closed DONE in
`feedback/2026-08-26-04-reviewer.md`. The seq-05/06 packet-provenance finding
was corrected by `f801b94b8`: the coder-created artifact falsely attributed to
the reviewer and its manifest block were deleted. The three V2-only fingerprint
gates identified in seq-05 route through `DistannEpochFingerprint::decode` in
this checkpoint. No reviewer authorship is claimed for any checkpoint-4
artifact.

This is the fourth and final format/lifecycle checkpoint, not a Task 229
closeout. It implements the one cataloged owner-local sidecar heap/index pair
and proves the covered generation's five-relation physical lifecycle. Read
selection, DML/recovery behavior, and the required 10k/50k/100k A/B remain for
packets 003 and 004.

## Implemented

- A covered generation owns exactly one bounded heap plus one unique directory
  index in addition to row tier, graph store, and graph directory. The
  generation catalog stores both sidecar OIDs. Creation uses the control
  owner's namespace/owner, the row tier's tablespace, permanent persistence,
  `fillfactor=100`, plain payload storage, and internal dependencies on the
  control index. Exact begin replay preserves the same five relation OIDs.
- Each handoff entry derives one compact sidecar payload from the already
  decoded canonical row values and inserts it in the same batch transaction as
  graph/row-tier state. The sidecar key is the requested row TID and stores the
  same `vec_id`; seal scans in canonical TID order, rejects row-count or
  row/graph identity divergence, computes the initial-content digest, and
  records heap/index bytes.
- The covered Ready receipt is now 351 bytes. Reviewer X14 was resolved by
  removing the redundant sidecar row-count field: a covered generation has
  exactly one initial sidecar row per owned record, so
  `owned_record_count` is the single receipt count. The physical scan still
  verifies the count, topology still reports it, and the global digest binds
  each owner's `owned_record_count` plus initial-content digest.
- The global sidecar initial-content digest is a manifest
  identity/consistency check against the immutable Ready receipts. It is not a
  later integrity check over the mutable sidecar heap and is never recomputed
  from live relation contents after DML.
- Publication accepts the covered V3 manifest/fingerprint and verifies the
  exact stored V2 Ready receipt. Generation read, handoff identity, and
  traversal-replica identity all use the canonical dual-version fingerprint
  decoder instead of hand-written V2-only prefix checks.
- Abort, cancelled-generation reclaim, and retired-generation reclaim pass all
  five OIDs to the shared relation-drop path. Publication and retirement retain
  all five relations; a forced transaction rollback restores all five; final
  reclaim drops all five transactionally and leaves the durable reclaim
  tombstone. Exact reclaim replay remains idempotent.
- No-cover generations retain their existing three-relation topology and
  frozen V2 descriptor / V1 receipt / V2 manifest and fingerprint bytes.

## Validation

- `cargo fmt --all -- --check` — pass.
- `cargo check --lib --no-default-features --features pg18` — pass.
- Focused payload-sidecar codec tests — 6/6 pass.
- Covered identity/receipt filters — 4/4 and 1/1 pass.
- Frozen on-disk DistANN fixtures — 21/21 pass.
- `cargo pgrx test pg18 test_distann_cover_sidecar_lifecycle` — 1/1 pass;
  proves creation shape/dependencies, begin replay, canonical sidecar handoff,
  seal evidence/topology, and five-relation abort.
- `cargo pgrx test pg18 test_distann_cover_sidecar_retire_reclaim_rollback` —
  1/1 pass; proves covered V3 publish, retention through publish/retire,
  five-relation rollback, final reclaim, tombstone state, and replay.
- Strict all-target clippy reports exactly the four inherited main failures:
  `collapsible_if`, `unnecessary_unwrap`, `needless_range_loop`, and
  `items_after_test_module`. After allowing only those four lint names, all
  targets pass under `-D warnings`; the task-local dead-code warning found on
  the first checkpoint-4 run was fixed in `c14c796b8` before this request.

All commands used the host's shared `CARGO_TARGET_DIR=/home/peter/.cargo-target`.
No custom target, new worktree, corpus, benchmark fixture, or benchmark cluster
was created. Durable command output is listed in `artifacts/manifest.md`.

## Deliberately remaining for Task 229

- Packet 003: fail-closed Task-222 projection selection; byte-identical
  fallback for uncovered/whole-row/unsupported queries; covered reads; Task-167
  insert/replacement/delete atomicity; restart/outage and the full correctness
  matrix.
- Packet 004: checked-in `ecaz bench suite` config and matched-position or
  counterbalanced cover-off/on evidence at 10k/50k/100k for recall, latency,
  storage, build, DML, owner stages, reads, and bytes; explicit PROMOTE or STOP.

## Review questions

1. Does checkpoint 4 implement exactly one bounded owner-local lookup relation
   pair and bind it to the existing generation without introducing a second
   sidecar variant?
2. Are build/handoff/seal/publish/retain/retire/reclaim/abort and transactional
   rollback complete for all five generation relations, including exact replay?
3. Is removing the redundant receipt sidecar count correct, with
   `owned_record_count` now the sole canonical count while physical/topology
   validation remains intact?
4. Do the two focused PG18 tests directly prove the lifecycle claims needed to
   close packet 002 and authorize packet 003 correctness/DML work?
