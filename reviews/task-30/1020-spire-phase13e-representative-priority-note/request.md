# Review Request: SPIRE Representative Priority Note

Task: `plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`

Code commit: `83a24719577653257028a41cf83c7f75e4d98d78`

## Summary

This doc-only checkpoint updates the Phase 13e task note so the durable plan
matches the current execution priority:

- representative pooling latency and recall proof remains the next AWS lane;
- fault rerun / resilience evidence remains deferred until after that packet;
- the local hardening range now includes packet `1019`;
- packet `1019` is called out as the verifier change that prints the accepted
  suite-driven nprobe sweep for packet-local AWS evidence.

## Validation

- `git diff --check -- plan/tasks/task30-phase13e-spire-aws-production-gap-closure.md`
- Runtime tests were not run; this checkpoint only updates task-state prose.

## Artifacts

- `artifacts/manifest.md`
- `artifacts/git-show-stat.log`
