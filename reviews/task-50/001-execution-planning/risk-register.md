# Task 50 Risk Register

| Candidate | Failure mode | Mitigation | Verification |
| --- | --- | --- | --- |
| AM callback helper | Closure or function-pointer wrapper inhibits inlining or changes panic/error boundary shape. | Keep helper `#[inline]`; preserve `pgrx_extern_c_guard` call shape; start with one IVF callback in Slice 1a. | Compile/lint; block-count delta; local same-host bench only if a hot callback path changes; optional disassembly/codegen check for hot users. |
| IVF page tuple visitor | Visitor changes tuple bounds, item-id interpretation, or error ordering. | Keep immutable and mutable visitors separate; encode line-pointer chain once; start with one tuple family before posting-range traversal. | Focused runtime tests when tuple selection or error text can drift; local IVF/RaBitQ before/after for iteration; AWS comparison for closeout. |
| SPIRE ActiveEpochAnchor | Type hides lock acquisition or reorders root-control/manifest/placement validation. | Keep lock decisions in callers or existing guards; anchor proves data chain only; split 3a/3b/3c. | Local SPIRE read-efficiency smoke during iteration; AWS SPIRE baseline before 3c closeout; snapshot diagnostics tests for row-shape stability. |
| Heap source scorer | Accidental per-candidate allocation or vector copy regresses rerank. | Borrow where possible; reuse slot and scratch buffers; keep allocation out of per-candidate loop. | Local IVF/RaBitQ and SPIRE before/after benches plus `dhat` smoke; AWS confirmation for closeout. |
| Reloption wrapper | Offset/C-string wrapper changes default parsing or planner-visible options. | Round-trip existing `build_local_reloptions` layout; keep AM-specific defaults unchanged. | Compile/lint; focused option parse tests; no bench unless planner/cost behavior changes. |
| WAL + exclusive buffer pair | Helper broadens mutation lifetime or calls `finish()` at the wrong point. | Closure API ties registered page image and mutation scope; do not combine with page visitor rollout. | Page mutation tests; crash/WAL tests if behavior can drift; block-count delta. |
| Vector datum wrapper | Wrapper copies detoasted vector data unnecessarily or relaxes layout validation. | Expose dimensioned borrowed slice where lifetime permits; copy only when the existing path copied. | Local `quant_encode` and `dhat encode` smoke plus local IVF/RaBitQ before/after; AWS confirmation for closeout. |
| SIMD load/store newtypes | AVX2/FMA and NEON paths diverge in inlining or target-feature handling. | Keep `#[target_feature]` on existing kernel functions, not newtype methods; wrappers remain lane-local and inline. | Required local x86_64 AVX2/FMA measurement plus cloud Graviton NEON measurement. |
| DSM atomic field wrapper | Typed wrapper accidentally changes memory-ordering or DSM layout assumptions. | Preserve exact PostgreSQL atomic primitive and layout; add static size/offset checks if possible. | Parallel build slot tests; Task 40 coordination; HNSW build latency if hot path is touched. |

## Review Rule

Every implementation packet should cite the relevant row above and state which
mitigation and verification path was used. If a slice introduces a new failure
mode, add it here before requesting review.
