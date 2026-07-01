# Task 130: TurboQuant Post-Task-124 Cleanup

Status: **review requested — clean main-based branch** (2026-06-30; packet `reviews/task-130/001-clean-main-keep-set/`).
Owner: coder (to be assigned). One coder, one branch.
Priority: P1 before any Task 124 landing or closeout PR merge.

## Why

Task 124 produced useful TurboQuant work and useful negative evidence, but the
full Task 124 branch is not a clean landing branch. It contains recall-broken
experimental IVF formats (`turboquant_binary`, `turboquant2`,
`turboquant2_768`) and the TQ2-only `qjl2_32` scorer module. Those formats were
valuable for measurement, but they failed the recall contract and must not land
as product-facing reloptions.

The validated production result is narrower:

- production 4-bit `turboquant` improved in-engine scorer elapsed by `-5.4%` at
  100k through peripheral scorer-path improvements;
- memory-traffic reductions around the TQ stage-2 path were kept;
- TQ2 / binary / reduced-dimension variants remain negative evidence only.

Task 130 curates the landing branch from current `main` so the code diff is the
keep-set only, not the full experimental Task 124 history.

## Goal

Land a clean TurboQuant cleanup branch that:

- keeps the validated production 4-bit TQ stage-2 pipeline and scorer-path
  speedups;
- removes all Task 124 recall-broken IVF experimental formats from callable
  source;
- preserves negative evidence in Task 124 packets and Task 130 documentation;
- fixes review-packet truth-cache ignore hygiene.

## Required Keep Set

- 4-bit `rerank_format=turboquant` IVF stage-2 final rerank pipeline.
- Stage-2 attribution counters.
- Selected TQ payload loader and contiguous selected-payload slab.
- Group-width locality control for the retained 4-bit TQ path.
- No-QJL gamma elision where the payload does not need gamma.
- LUT16 query-prep improvement.
- Batch payload cascade improvements for production TQ no-QJL and QJL paths.
- TQ LUT32 and prefetch profiler harnesses that do not introduce callable
  recall-broken formats.
- Phase-6 macOS relation-cache eviction CLI fix.
- `.gitignore` coverage for regenerable `truth-*.json` files.

## Required Prune Set

The landing branch must not contain these Task 124 IVF surfaces:

- `RerankFormat::TurboQuantBinary`;
- `RerankFormat::TurboQuant2`;
- `RerankFormat::TurboQuant2Dim768`;
- `turboquant_binary`, `turboquant2`, `turboquant2_768`, `tq2`, `tq2_768`
  IVF rerank-format parsing aliases;
- `src/quant/qjl2_32/`;
- TQ2 or binary IVF encode/rerank/scan dispatch.

Pre-existing HNSW `turboquant_binary` runtime code on `main` is out of scope for
this cleanup; Task 130 only prunes the Task 124 IVF formats that would otherwise
be introduced by the landing branch.

## Acceptance Criteria

1. Branch is based on current `origin/main`, not the full Task 124 branch.
2. `git diff origin/main...HEAD` contains the keep-set only and no IVF enum
   7/8/9 recall-broken formats.
3. Source search finds no `TurboQuant2`, `TurboQuantBinary`, `qjl2_32`,
   `turboquant2`, or IVF `turboquant_binary` additions outside pre-existing
   HNSW code.
4. Build, clippy, focused tests, and a 4-bit `turboquant` recall smoke are run
   or any blocker is documented in the Task 130 packet.
5. Review-packet truth JSON files are ignored by `.gitignore`.

## References

- `plan/tasks/124-ivf-tq-stage2-rerank-pipeline.md`
- `reviews/task-124/007-tq-binary-stage2-suite/`
- `reviews/task-124/008-tq2-stage2-suite/`
- `reviews/task-124/035-post-scorer-product-suite/feedback/2026-06-30-03-reviewer.md`
- `reviews/task-124/036-tq2-real-index-validation/`
- `reviews/task-124/037-tq2-dim768-real-index/`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
