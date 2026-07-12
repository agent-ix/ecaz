---
agent: codex
role: coder
model: gpt-5
date: 2026-07-11
seq: 01
---

# Review request — Packet 024 audited predecessor abandonment

This checkpoint adds the explicit binding-specific operator escape hatch for a
permanently unavailable predecessor owner after successor activation.

## Commit

- `3eabc6aab` — audit predecessor abandonment.

## Behavior

- `ec_distann_abandon_predecessor_binding(regclass, uuid, integer, text)` locks
  the exact predecessor disposition and covering successor decision under the
  coordinator control lifecycle lock.
- Only a `Pending` binding covered by an `Activated` successor can transition.
  The endpoint constructs and digest-validates canonical
  `DistannAbandonBindingAuditV1` bytes from owned immutable binding/decision
  identities plus session caller, timestamp, and bounded nonempty reason.
- The disposition changes atomically to `Abandoned` with the audit bytes/digest.
  The successor advances to `Applied` only when no binding remains `Pending`.
- No participant lifecycle endpoint is contacted and no remote reclaim is
  claimed. The unavailable participant may truthfully remain Published.
- Exact replay with the same target/reason succeeds. A different reason or a
  Retired/non-Activated target fails `EC_PREDECESSOR_ABANDON` without mutation.

## Live evidence

The PG18 four-epoch fixture activates a successor, abandons its sole pending
predecessor binding, verifies the durable audit and `Applied` covering decision,
and verifies the uncontacted predecessor generation remains `Published`.

Validation and provenance are in `artifacts/manifest.md`.

## Explicit next work

- Remote build, publication, retirement, and read dispatch plus the real
  three-node physical fixture remain the dominant implementation gap.
- Bounded persisted head seeds and required 10k/50k/100k measurements remain
  open.

Leaving this request open for outside review.
