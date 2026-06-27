# Task 114: IVF Dense Block Lifecycle and Repack Policy

Status: **proposed**.
Priority: P1 operational follow-up after Task 111.

## Goal

Make scan-dense IVF posting blocks practical under inserts, deletes, updates,
and vacuum without losing the scan-density benefit over time.

Task 111 may introduce immutable dense blocks for build-time data. This task
owns the lifecycle policy that keeps those blocks correct and decides when
sparse blocks should be repacked, reclaimed, or left for a rebuild.

## Why

A dense block is fast because it stores many candidates as one physical scan
unit. Deletes and updates create holes inside that unit. If holes accumulate,
the scanner either scores dead lanes or pays masking/filtering overhead while
getting fewer live candidates per page.

The system needs an explicit policy for:

- representing dead lanes,
- routing new writes,
- merging frozen and delta data at scan time,
- vacuuming or reclaiming dead space,
- measuring density,
- and deciding when local repack or full rebuild is the right answer.

## Scope

- IVF dense blocks introduced by Task 111.
- Per-block live/dead state.
- Existing row-shaped appendable/delta postings for new inserts and mutable
  tail data.
- Scan merge of frozen dense blocks plus row/delta postings.
- Vacuum density metrics and reclaim rules.
- Churn stress coverage.

## Non-Goals

- Do not implement a background worker.
- Do not introduce automatic full-index rewrite without an explicit gate.
- Do not change quantizer scoring math.
- Do not make dense blocks mandatory for all IVF indexes.
- Do not change SPIRE object lifecycle.

## Phases

### Phase 1 - Density Model

- Define live/dead representation for dense blocks.
- Define density metrics: live lanes, dead lanes, fully dead blocks, sparse
  blocks, delta row count, and delta-to-frozen ratio.
- Add diagnostics to expose these metrics for test fixtures and benchmark
  packets.

### Phase 2 - Delete and Update Semantics

- Mark deleted lanes without corrupting remaining lanes.
- Ensure updates are represented as delete old + insert new through the
  existing mutable path.
- Preserve snapshot visibility semantics.
- Add focused tests for repeated delete/update of candidates in dense blocks.

### Phase 3 - Insert and Delta Path

- Keep inserts on row-shaped delta pages or another explicitly mutable format.
- Ensure scan correctly merges frozen dense candidates and delta candidates.
- Preserve dedup semantics when a vector appears in frozen and delta surfaces
  across update boundaries.

### Phase 4 - Vacuum and Reclaim

- Extend vacuum to count dense-block dead lanes.
- Reclaim fully dead dense blocks/pages where safe.
- Decide whether partially sparse blocks are locally repackable or rebuild-only.
- Document WAL/crash-recovery requirements for any rewrite path.

### Phase 5 - Repack/Rebuild Decision Gate

Use churn benchmarks to decide policy:

- If sparse-block overhead is low, document rebuild-only guidance.
- If sparse-block overhead is material and local repack is safe, implement a
  gated repack path.
- If local repack requires broad WAL or concurrency work, split it into a
  separate task.

## Acceptance Criteria

1. Dense block live/dead state is correct under delete/update.
2. Inserts continue to work through a mutable path.
3. Scan merges frozen and mutable postings correctly.
4. Vacuum reports density and can reclaim fully dead blocks/pages where safe.
5. Churn stress tests cover insert/delete/update/vacuum.
6. The final packet states the durable policy: rebuild-only, local repack, or
   follow-up task.

## Evidence Requirements

Lifecycle packets must include:

- focused correctness tests,
- churn workload description,
- density before/after vacuum,
- latency before/after churn,
- recall before/after churn,
- index size before/after vacuum or rebuild,
- WAL/crash-safety notes if any page rewrite path lands.

## Dependencies and Coordination

- Depends on Task 111 for the dense block format.
- Coordinates with Task 37 if crash-recovery coverage needs to expand.
- Coordinates with Task 42 if format-version or upgrade behavior changes.
