---
task: 235
packet: 004-operator-recovery-closeout
agent: Codex
role: coder
model: gpt-5
date: 2026-08-25
seq: 01
---

# Task 235 operator-recovery closeout request

Please review the operator and NFR-014 operational disposition at
`b871d5481376df87c60ae486d68bb78519944c21`. This packet is a focused closeout
view over packet 003's clean release PG18 secure matrix.

The status-unavailable cell uses a real committed full coordinator XID while a
`pg_test`-only fault makes authoritative status unavailable. Two identical
operator reaper calls return `xid_status_unknown:operator_required`, retain the
owner intent at `prepare_requested`, leave no prepared transaction, and perform
no action. Once authoritative status is visible again, recovery converges to
`commit_local`; a duplicate call is empty. The implementation therefore does
not infer commit, rollback, or abandonment from intent state, age, or missing
status.

The same matrix proves the remaining operational cases:

- lost commit/rollback-prepared acknowledgements converge from nonterminal
  intent rows and duplicate recovery is empty;
- one-owner partial completion and a prepared GID with a missing intent row are
  explicit, coordinator-status-driven cases rather than guessed outcomes;
- saturating all 32 owner prepared slots returns the stable readiness hint,
  performs no logical mutation, and leaves an operator-recoverable fence; and
- routed tombstone owner death fails the first VACUUM and converges on explicit
  retry without source-map, prepared-xact, or nonterminal-intent residue.

This satisfies the Task 235 slice of NFR-014 operational readiness: the reaper
is explicit/operator-driven, unknown status stops deterministically, capacity
failure has an actionable `max_prepared_transactions` hint, and recovery output
is attributable to coordinator/target without conninfo or secret material.
Task 236 remains the accepted source for the broader TLS, secret, privilege,
and error-redaction requirements.

The coder recommendation is ACCEPT. Task 235 remains review-open until an
outside reviewer accepts packets 003 and 004. Please rule specifically on the
status-unavailable STOP behavior, missing-intent recovery, prepared-slot hint,
duplicate idempotence, and the NFR-014 operational mapping.
