# Task 13: Insert Throughput Decontention

Status: planned follow-up on `main`

Progress notes:
- Review `10045` already narrowed the metadata lock scope around duplicate-scan early exit.
- A5 completion makes the remaining live-insert hotspots explicit on `main`: metadata-page drift
  accounting on every successful insert, tail-page append contention, and repeated writes to
  popular neighbor pages during backlink mutation.

## Scope

Measure and reduce lock-driven contention in live graph-aware insert without weakening the
correctness rules in ADR-026.

## Subtasks

- [ ] **Contention baseline.** Measure current-main insert throughput and wait behavior on the
  contention-sensitive path after A5.
- [ ] **Metadata accounting decontention.** Revisit where and how `inserted_since_rebuild` is
  persisted so every successful insert does not necessarily serialize on the metadata page.
- [ ] **Tail-page hotspot reduction.** Evaluate append-page reuse vs allocation policy and any safe
  sharding/buffering options for the tail-page write path.
- [ ] **Backlink hot-page mitigation.** Evaluate whether immediate backlink writes can be reduced,
  delayed, or otherwise made less hotspot-prone while preserving graph reachability guarantees.
- [ ] **Validation harness.** Add a repeatable insert-contention benchmark/regression harness so
  later changes are measured instead of guessed.

## Owns

- Post-A5 optimization follow-up for `FR-016` / `NFR-001`

## Dependencies

- Task 06 (graph-aware insert) — complete

## Unblocks

- Higher-concurrency insert benchmarking
- Post-gate throughput optimization work

## Deliverables

- Measured insert-contention baseline
- One or more decontention checkpoints with validation evidence
- Documented decision on metadata-accounting strategy

## Notes

- Keep ADR-026 lock ordering intact unless a replacement design has a stronger deadlock argument.
- The expected first optimization target is metadata-page contention, not the physical lock order
  itself.
