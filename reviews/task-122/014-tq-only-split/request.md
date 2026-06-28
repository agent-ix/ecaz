# Task 122 Packet 014: TQ-Only Landing Split

This packet records the final split requested after Task 122 closeout:

```text
land only the TurboQuant evidence/docs/Task 124 handoff from Task 122
```

The SPIRE pre-materialization prune work explored on this branch is no longer
part of the Task 122 landing set. It can be recovered from branch history and
rolled into a separate SPIRE task/PR if desired.

## Removed From Task 122 Landing

- SPIRE source changes:
  - `src/am/ec_spire/options/mod.rs`
  - `src/am/ec_spire/scan.rs`
  - `src/am/ec_spire/scan/candidates.rs`
  - `src/am/ec_spire/scan/tests/diagnostics.rs`
- Shared comment-only source change:
  - `src/am/common/candidate_batch/mod.rs`
- SPIRE-only packets:
  - `reviews/task-122/002-spire-bounded-materialization-prune/`
  - `reviews/task-122/003-spire-batched-materialization-prune/`
  - `reviews/task-122/004-spire-prune-ab-suite/`
  - `reviews/task-122/005-spire-prune-release-suite/`
  - `reviews/task-122/006-spire-recall-width-sweep/`
  - `reviews/task-122/007-spire-latency-storage-width25/`

## Remaining Task 122 Landing Content

- Task 122 planning and closeout docs.
- Task 124 TurboQuant-focused follow-up.
- Packet 001 TurboQuant scorer inventory.
- Packet 008 TurboQuant sidecar final-rerank harness validation.
- Packet 009 TurboQuant stage-2 sidecar suite evidence.
- Packets 010-014 closeout, correction, and split metadata.
- CLI sidecar harness support for `turboquant4` and `final_rerank_k`.

## Rationale

The user clarified that Task 122 should have stayed focused on TurboQuant
competitiveness. The SPIRE prune may be valid work, but it is not the requested
TurboQuant optimization path. The landing branch therefore keeps the TQ-specific
evidence and the Task 124 handoff, while leaving SPIRE work for a separate
SPIRE-owned task.

## Validation

No new tests or benchmarks were run for this split packet. The split restores
SPIRE source files to `origin/main` and removes SPIRE-only packets from the
final branch tree.
