# Task 122: TurboQuant Bit-Mode SIMD Coverage

Status: **proposed**.
Owner: coder (to be assigned). One coder, one branch.
Priority: P0 only if it unblocks Task 89 / TQ+ or a current AM profile; P2
otherwise.

## Why

Task 119 measured the full HNSW RaBitQ-1 coarse-rerank matrix and found an
important split:

- `turboquant_4bit` was the best compact latency/storage lane, because the
  1536-dimensional surface uses the optimized tiled no-QJL 4-bit path.
- `turboquant_8bit` recovered much more recall, but the current measured
  scoring path was too slow.
- `turboquant_2bit`, `turboquant_3bit`, and `turboquant_5bit` through
  `turboquant_7bit` were measured, but they were dominated in that harness.

That result should not turn into speculative kernel work for every bit label.
The project already has optimized kernel families for the current production
surfaces: no-QJL 4-bit LUT/tiled LUT, QJL-active 4-bit, grouped PQ, RaBitQ, and
binary/Hamming. What is missing is a current, explicit inventory that maps every
TurboQuant bit/configuration that a live AM or Task 89 TQ+ path can actually
use to one of:

- optimized production batch kernel;
- scalar or per-candidate fallback;
- structurally absent / no current consumer;
- intentionally deferred because Task 89 or product evidence does not need it.

The immediate product question is TQ+. Task 89 is the main TQ+ validation task.
If TQ+ remains a no-QJL 4-bit calibration variant, then the core scoring path is
already covered by the optimized 4-bit surfaces and this task should close with
a stop-condition packet. If Task 89's ADR or port work expands TQ+ into
QJL-active or non-4-bit modes, this task owns the missing SIMD/dispatch gap
before those modes can be benchmark-promoted.

## Goal

Produce a reviewer-approvable TurboQuant SIMD coverage decision that answers:

1. Which TurboQuant bit/mode combinations are reachable by current AMs or by
   Task 89 TQ+?
2. Which reachable combinations route through optimized batch scoring on M5,
   Intel, and Graviton lanes?
3. Which missing kernels or dispatch hooks must land before Task 89 can make a
   production claim?
4. Which bit/mode combinations are not relevant and should remain
   structurally absent rather than optimized speculatively?

## Scope

### Phase 0 - Surface Inventory and TQ+ Relevance Gate

Audit current code and Task 89's intended TQ+ format design for every
TurboQuant mode:

- no-QJL 4-bit;
- QJL-active 2-bit through 8-bit labels, with explicit MSE/QJL composition;
- any TQ+ calibration mode restored from the Task 86 preserved commits;
- each AM surface: IVF, SPIRE, HNSW, DiskANN.

For each cell, record:

- AM;
- placement or storage path;
- bit label and actual MSE/QJL composition;
- query preparation path;
- scalar scorer path;
- batch scorer path;
- current kernel status: optimized, scalar fallback, per-candidate fallback,
  missing kernel, structurally absent, or retired;
- whether Task 89 TQ+ can reach the cell.

Stop condition: if TQ+ only needs the already-optimized no-QJL 4-bit lane and
no current AM exposes another product-relevant TurboQuant bit mode, close this
task with the inventory packet and do not implement kernels.

### Phase 1 - Dispatch and Counter Audit

For every reachable non-stop-condition cell:

- verify `candidate_batch` or the AM-local scoring path uses the optimized
  batch surface when candidate width permits;
- verify `kernel_status` / `(AM, quant, ISA)` counters distinguish optimized,
  scalar, missing, structurally absent, and retired cells;
- add missing counter markers before performance work.

Do not write a kernel until the dispatch/counter surface can prove whether the
kernel is being used.

### Phase 2 - Kernel or Dispatch Gap Closure

Only for cells that Phase 0 and Phase 1 classify as relevant:

- add the narrowest missing SIMD kernel, or wire an existing kernel into the AM
  path;
- keep scalar reference parity tests;
- add forced-backend tests where practical;
- prove byte-identical or tolerance-bounded scores against the scalar reference.

Likely candidate gaps, subject to Phase 0:

- QJL-active higher-bit TurboQuant paths if Task 89 or a real AM can use them;
- no-QJL 2-bit only if a real no-QJL 2-bit consumer exists;
- sidecar/rerank batch dispatch gaps if the kernel exists but the rerank path
  is still per-candidate.

### Phase 3 - Measurement

Run suite-driven measurements for every changed relevant cell:

- local M5 first;
- Intel and Graviton only if the cell is production-relevant or Task 89 needs
  cross-lane evidence;
- recall/result identity against the pre-change scorer;
- p50/p95/p99 latency;
- scoring-share counters;
- storage unchanged unless the task explicitly lands a format change, which is
  discouraged here.

Use `ecaz bench suite` for benchmark matrices. Do not write ad hoc sweepers.

### Phase 4 - TQ+ Handoff

Publish a final packet that explicitly states one of:

- Task 89 is not blocked by TurboQuant SIMD coverage because TQ+ uses already
  optimized no-QJL 4-bit scoring;
- Task 89 is unblocked after this task's specific kernel/dispatch fixes;
- Task 89 should not expand into the affected bit/mode combination until a
  separate format/product task justifies that surface.

## Non-Goals

- Do not implement kernels for bit labels with no current AM or TQ+ consumer.
- Do not re-open the Task 89 TQ+ format/product decision here.
- Do not change durable TurboQuant storage layout unless Task 89's ADR first
  requires it.
- Do not use Task 119's sidecar-rerank harness latency alone to justify broad
  production kernels; verify the relevant production AM path first.
- Do not pursue HNSW RaBitQ coarse-rerank promotion in this task.

## Required Evidence

- Inventory packet under `reviews/task-122/` with the full cell table.
- If no kernel work is needed, a stop-condition packet citing why Task 89 is
  not blocked.
- If kernel/dispatch work lands:
  - scalar-vs-SIMD parity tests;
  - forced-backend or host-backed validation where practical;
  - `ecaz bench suite` results with packet-local manifests;
  - before/after latency and counter evidence;
  - no recall/result identity regression.

## Acceptance Criteria

1. Every TurboQuant bit/mode combination reachable by current AMs or Task 89 is
   classified as optimized, scalar/per-candidate fallback, missing, retired, or
   structurally absent.
2. TQ+ relevance is explicit: Task 89 is either unblocked or blocked on a
   concrete missing kernel/dispatch cell.
3. No speculative kernel lands for a cell without a live consumer.
4. Any landed kernel or dispatch change has scalar parity, counter, and
   suite-driven benchmark evidence.
5. Final closeout recommends one of: stop/no-op, unblock Task 89, or file a
   narrower follow-up for a specific bit/mode surface.

## References

- `plan/tasks/89-turboquant-tqplus-cross-am-validation.md`
- `plan/tasks/96-tq-2bit-block-kernel-family.md`
- `plan/tasks/97-tq-qjl-block-kernel-family.md`
- `plan/tasks/98-hnsw-exact-mode-block-kernels.md`
- `plan/tasks/99-cross-am-quant-isa-block-kernel-closeout.md`
- `plan/tasks/119-hnsw-rabitq-coarse-rerank-profile.md`
- `reviews/task-119/007-final-closeout/`
- `spec/non-functional/NFR-007-benchmark-provenance.md`
