---
agent: claude
role: coder
model: claude-opus-4-8
date: 2026-07-11
seq: 01
---

# Review request — Packet 012 publish decision (T3)

Implements `ec_distann_decide_epoch_publish(index_regclass regclass, build_id uuid)
RETURNS bytea` (FR-082:251-280), the first coordinator publication endpoint, on
top of the integrated participant lifecycle (907150c03).

## Commit
- `2ada9f9a` — `ec_distann_decide_epoch_publish` + `load_build_candidate`.

Artifacts + provenance in `artifacts/manifest.md`.

## Contract mapping (FR-082:251-280)
- **Digest-chain recompute (266-270)**: `load_build_candidate` reconstructs the
  canonical `DistannBuildCandidateV1` from the stored candidate columns and
  `validate()` recomputes/verifies every component digest and the cross-component
  consistency before any mutation (mismatch → `EC_PUBLISH_DIGEST`).
- **Parent == active pointer (271-277)**: the `ec_distann_active_epoch` row is
  taken `FOR UPDATE` under the control `ShareRowExclusiveLock`; the candidate's
  build-spec parent fingerprint must equal it (empty iff absent).
- **Commit-only Pending decision (278-280)**: persists the decision with the
  predecessor tuple (NULL for a first epoch) and canonical
  `DistannSuccessorActivationV1` bytes/digest, `decision_state='Pending'`.
  Returns the 32-byte manifest digest. **No** participant publish call and **no**
  active-pointer swap — those are `ec_distann_recover_epoch_publish` (T4a).
- **Idempotent replay**: an existing decision returns the same manifest digest; a
  conflicting manifest for the same build id is rejected.

## Deliberately scoped / follow-ups
- **Live participant topology re-run** (FR-082:251-260): the candidate's
  `validate()` already re-establishes the receipt↔manifest↔descriptor
  consistency from the stored bytes; re-running `ec_distann_generation_topology`
  against each live participant to guard against post-build drift is a hardening
  follow-up (and becomes essential with the multi-owner/remote lane).
- Single/first-epoch tested end-to-end; the predecessor path (non-empty active
  pointer) is code-complete but exercised once `recover` (T4a) can swap a first
  pointer in a follow-up packet.
- Next: `ec_distann_recover_epoch_publish` (T4a publish + pointer swap → Activated;
  T4b mark predecessors Retired → Applied), then retire/abandon.

## Validation
- `cargo check` + strict clippy (`pg18 pg_test`, `-D warnings`) — pass at `2ada9f9a`.
- `cargo pgrx test pg18 test_distann_build_epoch_single_node` — 1/1 pass (build →
  Ready → decide → Pending decision, no swap, idempotent replay).

Leaving the request open for outside review.
