# Task 122 Packet 013: Task 124 Numbering and Focus Correction

This packet corrects the Task 122 follow-up handoff after the user clarified
two points:

- Task 123 is reserved externally, so the TurboQuant follow-up must be Task 124.
- The intended continuation is TurboQuant competitiveness work, not another
  SPIRE-centered optimization or measurement-only task.

## Correction

The follow-up task is now:

```text
plan/tasks/124-ivf-tq-stage2-rerank-pipeline.md
```

The former local `plan/tasks/123-ivf-tq-stage2-rerank-pipeline.md` was moved to
Task 124.

## TurboQuant Focus

Task 124 explicitly frames the problem as TurboQuant failing to beat the
current RaBitQ + f32 baseline on recall and latency, with latency the largest
blocker. The task now requires the next coder to stay on TurboQuant-specific
surfaces:

- verify the active TQ stage-2 scorer uses the intended SIMD/block path;
- begin with an explicit TurboQuant SIMD/scalar-surface audit;
- report scorer family, ISA, flush widths, scalar-tail count, and scorer time;
- implement the in-engine IVF stage-2 path, not a sidecar-only measurement;
- diagnose recall loss separately from candidate containment, final f32 width,
  TQ score quality, and implementation overhead;
- keep drilling into TQ-specific latency bottlenecks before closing.

The task file now has focus guardrails: SPIRE-only, RaBitQ-only, generic
materialization, and measurement-only work are out of scope unless they directly
block or explain the TurboQuant stage-2 path.

## Notes

The outside reviewer feedback in packet 010 still says "Task 123" because that
was the local follow-up number at review time. This packet supersedes only the
follow-up number and focus wording; it does not change the accepted Task 122
closeout outcome.

No code, tests, or benchmark evidence changed in this packet.
