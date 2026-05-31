---
task: 75
topic: phase2-decision-after-diagnostic-fix
agent: codex
role: coder
model: GPT-5
date: 2026-05-31
---

# Task 75 Phase 2 Decision After Diagnostic Fix

## Request

Please review the reissued Phase 2 decision after the diagnostic fix in `reviews/task-75/004-diagnostic-fix-rerun/`.

## Decision

No Task 75 routing slice is being shipped.

The corrected diagnostic changes the magnitude but not the action:

- The old high-recall `2,784,952` candidate plateau was wrong.
- The corrected high-recall tg96/tg128 envelope is `15,506,227` candidates over 200 queries.
- Recall rises with a larger candidate envelope: tg16 `0.8525`, tg32 `0.9310`, tg64 `0.9825`, tg96/tg128 `0.9975`.
- The tg96/tg128 plateau is explained by effective `nprobe=96`; tg128 does not add leaves beyond the resolved scan plan.

The mechanism behind the nprobe-to-recall curve is therefore straightforward: more routed leaves produce more scored candidates until the resolved high-recall cap. There is no evidence of a safe routing predicate that can drop a meaningful share of leaves before scoring while preserving recall.

## Slices Considered

| Slice | Decision | Reason |
| --- | --- | --- |
| Tighter recursive draft | Shelved | The active high-recall path is top-graph routing, and the corrected diagnostic shows recall is coming from expanded leaf coverage, not obviously dead recursive draft leaves. |
| Score-bound early termination | Deferred to Task 77 | This targets candidate materialization/scoring cost, not routing semantics. It needs a proof that omitted candidates cannot beat the heap bound. |
| Adaptive nprobe collapse | Shelved | tg128 collapses to effective nprobe96, but tg16->tg96 still trades latency for recall. Collapsing nprobe earlier would be a recall/defaults decision, not a semantics-preserving latency slice. |

## Follow-Up

Task 77 (`plan/tasks/77-spire-candidate-materialization-optimization.md`) now owns the optimization research that is still plausible after this rerun: score-bound early termination, cheaper discarded-candidate paths, leaf-local batching, and candidate replay microbenching.

## Evidence

- Diagnostic fix and rerun request: `reviews/task-75/004-diagnostic-fix-rerun/request.md`
- Benchmark packet: `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/manifest.md`
- Suite report: `benchmarks/task75-intel-local-routing-envelope-diagnostic-fix-rerun/artifacts/suite-report.md`
