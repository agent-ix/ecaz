# Task 121 Closeout Status Sync

## Scope

This packet records the canonical task-status update requested by the accepted
Task 121 closeout review:

- status update commit: `33d641b68 Update task 121 closeout status`;
- closeout packet: `reviews/task-121/026-phase4-final-pareto-verdict/`;
- reviewer sign-off:
  `reviews/task-121/026-phase4-final-pareto-verdict/feedback/2026-06-26-01-reviewer.md`;
- files updated:
  - `plan/tasks/121-spire-coarse-routing-recall-doe.md`;
  - `plan/tasks/README.md`.

No code changed and no new benchmark was run in this packet.

## What Changed

The task file and task index now say **complete - evidence-backed no-promote /
wall result** instead of **Phase 0 tooling under review**.

The update records the accepted closeout findings:

- route-stage containment equals final recall in every measured run;
- boundary replication is the primary route-recovery lever;
- the practical `b4/tr50/f8` candidate is not a default because the storage and
  low-nprobe latency cost slope is too steep;
- b8 proves saturation but is a storage/latency wall;
- retuned sampled block pruning is neutral at the low/mid operating point and
  only helps at high nprobe;
- the reviewer loose thread on pruning-as-I/O is preserved: object bytes stayed
  unchanged while candidates dropped, so any future I/O-pruning claim must prove
  whether pruning can avoid reads rather than only post-read compute.

## Requested Reviewer Decision

Please confirm the status sync accurately reflects the packet 026 sign-off and
the current Task 121 state.
