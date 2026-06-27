# Task 117: IVF F16 Index-Only Rerank Revival

Status: **proposed / do not start unless f16 is deliberately revived**.
Priority: P2 optional compact rerank follow-up after Task 111h and Task 116.
Source: Task 111h closeout residual reviewer note
`reviews/task-111h/048-final-closeout-decision-v9/feedback/2026-06-20-01-reviewer.md`.

## Goal

Revisit f16 rerank only under a design that can beat source/f32 on a real product
axis: storage, cold I/O, or index-only serving.

The task is not "make the current duplicated f16 sidecar slightly faster." The
current Task 111h conclusion is that f16 sidecar rerank is structurally
dominated when it duplicates an existing f32 source column: same recall, more
index bytes, more cold I/O, and no storage win. f16 is worth reopening only if
it replaces the source f32 payload for the rerank path or otherwise removes a
baseline storage/read cost.

## Why

Task 111h verified that the fixed f16 query path is no longer doing the old
per-query map rebuild or per-candidate f32-to-f16 conversion. The remaining
latency loss is primarily storage duplication and I/O, not an obvious scoring
bug.

There is still a legitimate f16 opportunity if the architecture changes:

- store the durable rerank payload as f16 instead of duplicating f32,
- avoid query-time conversion per row,
- convert the query side once per scan if needed,
- score batches directly from persisted f16 payloads,
- show that the lower byte footprint improves cold or memory-resident behavior.

That design must be common and format-aware, not a one-off f16 exception.

## Scope

- IVF coarse rerank only.
- f16 as a durable rerank payload that can replace or avoid an f32 storage/read
  cost.
- Common rerank storage integration from Task 116.
- Scalar reference scorer plus SIMD/batched scorer where available.
- Recall, latency, storage, and build/load evidence against source/f32.
- Explicit table/index storage semantics: what bytes are added, removed, or
  reused.

## Non-Goals

- Do not duplicate the f32 source column and call f16 compact.
- Do not convert every candidate vector from f32 to f16 at query time.
- Do not add a separate f16-only sidecar architecture.
- Do not promote f16 unless it beats source/f32 on storage-adjusted evidence.
- Do not hide table storage growth outside index-size accounting.

## Phases

### Phase 1 - Storage Contract

- Define whether f16 replaces the table/source rerank payload, lives in the
  index, or uses another explicit storage surface.
- Account for total bytes: table bytes, index bytes, WAL/build bytes, and any
  auxiliary sidecar bytes.
- Define upgrade/rebuild behavior and metadata versioning.
- Confirm how heap-visible f32 values, if any, remain available for SQL semantics
  outside rerank.

Stop condition: if f16 cannot remove or avoid an f32 read/storage cost, close the
task without implementation and keep Task 111h's do-not-promote decision.

### Phase 2 - Common Storage Integration

- Use the Task 116 logical group and segment model for index placement.
- Do not add an f16-specific metadata layout.
- Ensure scan can borrow persisted f16 payloads without per-candidate allocation
  or source-vector materialization.
- Convert query-side data once per scan, not once per row.

### Phase 3 - F16 Batch Scoring

- Keep an exact scalar reference path for portability and tests.
- Add a batched scorer for f16 payloads on architectures where it is worthwhile.
- Prove scalar and batched paths agree within a documented tolerance.
- Preserve dispatch behavior when SIMD is unavailable.

### Phase 4 - Correctness Coverage

- Cover normal values, signed zero, subnormal magnitudes, inf/NaN handling if the
  surrounding vector contract permits them, and dimension tails.
- Cover build, insert, update/delete visibility, and vacuum when f16 is persisted
  index-side.
- Prove source/f32 and f16 rerank ordering differences are measured and expected,
  not silent data corruption.

### Phase 5 - Benchmark Gate

Run an `ecaz bench suite` matrix before any promotion claim:

- 10k, 50k, 100k, and 1M,
- warm and cold/cache-state labels where relevant,
- source/f32 baseline,
- best current compact rerank candidate from Task 111h,
- f16 scalar and f16 batched where both exist,
- storage accounting including table and index bytes.

## Acceptance Criteria

1. The packet states an explicit storage contract and byte accounting model.
2. f16 does not duplicate source/f32 storage unless the packet closes the task as
   not worth pursuing.
3. Query-time conversion is bounded to query-side data, not per-candidate row
   conversion.
4. f16 scoring has scalar parity coverage and SIMD/batch coverage if implemented.
5. Correctness tests cover edge values and mutation semantics for the chosen
   storage placement.
6. Benchmark evidence compares recall, latency, and total storage against
   source/f32 and the best compact alternative across 10k/50k/100k/1M.
7. The final packet recommends promote, iterate, or abandon based on measured
   value over source/f32.

## Evidence Requirements

Review packets must include:

- suite config and command lines,
- head SHA and storage contract,
- table bytes, index bytes, and total bytes,
- f16 payload layout and group layout,
- whether SIMD batch scoring was active,
- recall@10/NDCG@10,
- p50/p95/p99 latency with cache-state labels,
- build/load time,
- edge-value and mutation test logs,
- comparison tables against source/f32 and the best compact candidate.

## Dependencies and Coordination

- Depends on Task 116 if index-side f16 placement is pursued.
- Coordinates with Task 111h closeout evidence; do not reopen that decision by
  assertion.
- Coordinates with Task 42/NFR-016 for any persisted format-version change.
- Coordinates with future RQ8/TQ compact rerank work through the shared storage
  and benchmark surfaces.
