# Task 200: Task 38 Long-Running Interrupt-Poll Follow-ups

Status: **proposed by the Task 38 Apple-M5 audit** (2026-07-26). Priority: P1
correctness and operability.

## Why

Task 38 requires an inventory of `CHECK_FOR_INTERRUPTS` coverage in every
long-running ECAZ loop, with missing sites filed as follow-ups. The exact
current poll sites are now documented in `docs/hardening.md`, but source
inspection found long-running or potentially long-running surfaces whose
interrupt behavior is absent or depends on an outer PostgreSQL callback.

This task owns the exhaustive classification and any required remediation. It
must not add an interrupt check inside a Rust borrow/guard region where
PostgreSQL longjmp would skip destructors; the DistANN transport boundary is
the model for unwind-safe placement.

## Initial Audit Scope

- HNSW eager `amrescan` graph traversal and sequential build/page walks.
- IVF eager `amrescan` candidate collection/rerank, sequential build, and
  diagnostic/admin page walks.
- DiskANN build/import/page-flush loops not already covered by scan/vacuum
  `maybe_check_for_interrupts()` calls.
- SPIRE local build/scan CPU loops outside the bounded remote-dispatch
  cancellation future.
- DistANN legacy/local build loops and eager orchestration outside physical
  shard-build and remote-transport interrupt boundaries.
- Any additional five-AM long-running loop found by the audit.

## Required Classification

For every candidate loop, record exactly one outcome:

1. an explicit backend-safe interrupt poll is present;
2. the loop is bounded small enough that an internal poll is unnecessary,
   with the bound and outer PostgreSQL interrupt boundary documented;
3. an unwind-safe poll is added and covered by live cancellation/timeout
   evidence; or
4. a narrower follow-up task is filed because safe placement requires a
   structural refactor.

## Acceptance Criteria

1. A packet-local inventory maps every production loop in the five AM trees
   that can scale with rows, pages, candidates, graph nodes, shards, or remote
   participants to one required classification.
2. Every added poll sits outside Rust borrows/guards that PostgreSQL longjmp
   could strand.
3. Focused PG18 cancellation and statement-timeout probes demonstrate prompt
   termination for every remediated surface and clean shared postconditions.
4. `docs/hardening.md` is updated from the audit, and no surface remains
   silently “unknown.”
5. Any deferred structural work has its own task identity and explicit owning
   source paths.

## References

- Task 38 and `reviews/task-38/005-apple-m5-handoff/`.
- `docs/hardening.md` § “PG Fault Injection” interrupt inventory.
- DistANN transport longjmp-safety comments in
  `src/am/ec_distann/remote_transport.rs`.

