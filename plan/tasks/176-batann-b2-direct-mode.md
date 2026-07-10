# Task 176: BatANN B2 — Direct Return: Shmem Mailbox, Deliver Endpoint, Flush Spike

Status: proposed (2026-07-09). Depends on: Task 175 (incl. its recorded
proceed verdict).
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 — second return mode; gated by the B1 kill-check.

## Why

FR-088: the return path that frees intermediate backends. First-of-kind
machinery in this codebase (no existing shmem/latch/LwLock usage in
`src/`): fixed-slot shmem mailbox, monotonic query_id allocator, raw
`SetLatch`/`WaitLatch` against the waiter's procLatch, xact-abort cleanup,
`_PG_init` shmem wiring — plus the send-and-abandon flush question that
decides FR-088's normative shape.

## Goal

Direct-mode reads equal to stack mode; at-most-once delivery with the
timeout backstop; recorded spike verdict (send-and-abandon vs direct-lite).

## Scope

- **Pre-implementation timeboxed flush spike** (tokio-postgres lazy
  futures / no flush signal — direct-lite is the probable shipped form);
  verdict recorded against ADR-086 D4 before the mailbox work proceeds.
- Fixed-slot shmem mailbox + `ec_distann_relay_mailbox_status()`; 64-bit
  monotonic query_id allocator (never reused per postmaster lifetime).
- `ec_distann_deliver_result` (fingerprint + structural validation,
  first-delivery-wins, WARNING drops, oversize error, EXECUTE revoked from
  PUBLIC per D11); primary-only posture.
- Delivery-rights rule (no delivery after confirmed handoff; nothing on
  indeterminate outcomes); wait timeout = non-retriable classified error;
  slot-exhaustion transparent coordinator-mode fallback.
- Forwarding per spike verdict (send-and-abandon with busy-until-drained +
  cap-degrade + evict-on-error, or direct-lite sync acks).
- Mailbox lifecycle drills co-located here (TC-047 happy paths + slot
  lifecycle rows).

## Required Evidence

TC-047 happy-path + lifecycle drills green; stack ≡ direct result check;
spike verdict + chosen variant recorded (the NFR-022 gate packet later
reports the variant).

## Non-Goals

Cross-cutting fault matrix (177); gate matrix (178).

## Acceptance Criteria

1. FR-088 ACs 1–3, 5–7 green (AC-4's kill drill completes in 177 if the
   fixture hold hook lands there).
2. Zero leaked slots via `ec_distann_relay_mailbox_status()` after drills.
3. Spike verdict recorded in ADR-086 before B3 starts.

## References

- FR-088; ADR-086 D4/D10/D11; NFR-021 (mailbox budget, envelope-sized cap)
- `plan/design/batann-state-passing-coordination.md` (B2)
