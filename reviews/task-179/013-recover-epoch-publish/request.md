---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 013 publish recovery (T4a)

Implements `ec_distann_recover_epoch_publish(index_regclass regclass, build_id uuid)
RETURNS bytea` (FR-082:604-645), the T4a publish/activate step, completing the
single-node first-epoch publish pipeline (build -> decide -> recover -> Published).

## Commit
- `9f8cd1d4` — `ec_distann_recover_epoch_publish`.

Artifacts + provenance in `artifacts/manifest.md`.

## Contract mapping (FR-082:604-645)
- Loads the durable Pending decision (absent -> `EC_EPOCH_STATE`), publishes the
  local participant via `ec_distann_publish_epoch`, and verifies the acknowledged
  34-byte fingerprint equals the decided one.
- Atomically inserts the `ec_distann_active_epoch` pointer to the successor (taken
  `FOR UPDATE`), marks the decision `Applied` (no predecessor -> T4a may record
  Applied), and moves the registration to `Published`, which **clears the durable
  build gate** (Published is not a gated state). Session-lock release is scheduled
  from the post-commit callback (FR-082:638).
- Returns the 34-byte active epoch fingerprint. Idempotent: an active pointer
  naming this build returns the fingerprint without re-publishing; a different
  build id is rejected.

## Deliberately scoped / follow-ups
- **First-epoch (no-predecessor) single-node only.** A decision carrying a
  predecessor fails closed (`multi-epoch publish recovery (T4b) is not yet
  implemented`). T4b (mark predecessors Retired -> Applied) and the predecessor
  pointer CAS land with the multi-epoch slice.
- Next: retirement (`retire_epoch`/`recover_epoch_retire`/`force_retire_epoch`),
  `abandon_predecessor_binding`, then `epoch_topology` (by fingerprint, now that a
  Published/active generation exists) and the physical read path.

## Validation
- `cargo check` + strict clippy (`pg18 pg_test`, `-D warnings`) — pass at `9f8cd1d4`.
- `cargo pgrx test pg18 test_distann_build_epoch_single_node` — 1/1 pass (full
  build -> decide -> recover -> Published pipeline, gate cleared, idempotent).

Leaving the request open for outside review.
