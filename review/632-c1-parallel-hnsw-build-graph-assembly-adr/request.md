# Review Request: Parallel HNSW Build Graph Assembly ADR

Current head: `f851266`

Scope:
- `spec/adr/ADR-048-parallel-hnsw-build-graph-assembly.md`

Problem:
- Packets `626` through `631` show that the current task 19 branch has a
  working PG18 parallel heap-ingest path, but wall-clock build time is still
  dominated by serial native HNSW graph assembly.
- The existing `build_parallel` coordinator is correct for heap ingestion and
  encoded tuple transport, but it does not give graph workers a durable shared
  corpus or a writable graph surface.
- The scan coordinator is also the wrong abstraction for build graph assembly:
  it owns query/rescan/runtime state, not build corpus storage, graph patches,
  or deterministic page-staging input.

Decision Proposed:
- Add ADR-048 as the next design checkpoint for FR-021 graph assembly.
- Keep the current build coordinator for heap ingestion.
- Introduce a separate graph-assembly phase based on:
  - shared immutable build corpus
  - deterministic node partitions
  - worker-built local graph patches
  - leader-owned deterministic boundary merge
  - the existing `Vec<HnswBuildNode>` page-staging contract
- Reject shared concurrent HNSW insertion as the first implementation path
  because PostgreSQL workers are processes, not threads, and a DSM mutable graph
  would introduce fine-grained locking and nondeterministic insertion order
  before proving recall quality.

Validation:
- Design-only checkpoint.
- Ran `git diff --check`.
- No runtime tests or SQL measurements were run because this packet only adds
  an ADR.

Review focus:
- Whether partitioned local graphs plus deterministic leader merge is a credible
  next implementation path for parallel build, or whether shared mutable graph
  insertion should be pursued first despite its lock/determinism cost.
- Whether ADR-048 is explicit enough that the existing heap-ingest coordinator
  remains useful but is not the graph-assembly coordinator.
- Whether the proposed first spike is narrow enough: planning surface,
  partition planner, leader-local graph-patch simulation, then recall/speed
  validation before worker wiring.
