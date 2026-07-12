---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 010 coordinator build status

Implements `ec_distann_epoch_build_status(index_regclass regclass, build_id uuid)
RETURNS TABLE(...)` (FR-078:80-83), completing the coordinator inspection/teardown
trio (`build_epoch` pending, `abort_epoch_build` in packet 009, `epoch_build_status`
here).

## Commit

- `c36bbfa6b` — `ec_distann_epoch_build_status`.

Artifacts + provenance in `artifacts/manifest.md`.

## Contract mapping (FR-078:80-83)

Returns one row per registered roster participant with the exact spec columns:
`epoch, build_state, publish_decision_state, node_id, participant_state,
next_batch_seq, record_count, receipt_digest, last_error_category`.

- **epoch + build_state** from the registration; an absent build id yields **no
  rows** (tested).
- **publish_decision_state** from `ec_distann_publish_decision` when a decision
  exists, else NULL.
- **Per-participant fields** from the local participant's live generation row
  (`participant_state`, `next_batch_seq`, `record_count`). Before `build_epoch`
  creates a generation these are NULL — the participant is registered but has no
  generation yet (tested).
- **receipt_digest**: the SHA-256 content digest of the encoded Ready receipt
  (`domain_digest("ec_distann_ready_receipt_v1\0", receipt)`) when the generation
  is Ready, else NULL. (Flagging the exact `receipt_digest` semantics for
  reviewer confirmation — the spec column is untyped beyond `bytea`.)
- **last_error_category**: NULL — there is no durable error-tracking column yet;
  this is a forward-compatible placeholder.
- Read-only: opens the control index at `AccessShareLock`; derives everything
  from durable catalog rows.

## Deliberately scoped

A multi-node roster **fails closed** (`EC_BUILD_STATE: remote participant status
is not yet implemented`) rather than reporting misleading NULLs for unreachable
remote participants. Remote participant status (live RPC) lands with the
remote-transport slice. The generation-present participant rows (Building/Ready)
are exercised end-to-end once single-node `build_epoch` lands and ties the
registration to a generation; this packet tests the registration-level view.

## Validation

- `cargo check` + strict `cargo clippy` (`pg18 pg_test`, `-D warnings`) — pass at
  `c36bbfa6b`.
- `cargo pgrx test pg18 test_distann_epoch_build_status_registration` — 1/1 pass.

Leaving the request open for outside review.
