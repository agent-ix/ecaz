# Task 86: TurboVec-Derived TurboQuant Improvements

Status: coder-complete pending reviewer acceptance (2026-06-07)
Owner: coder (to be assigned). One coder, one branch.
Priority: 1 (TurboQuant scan/storage improvement lane across AMs)

## Why

TurboVec claims a TurboQuant-based path that scans compressed vectors directly,
keeps vectors smaller than FAISS-style baselines, and avoids decompressing
database vectors at query time. The relevant question for this repository is
not whether TurboVec beats FAISS or another quantizer. The relevant question is
which implementation choices in TurboVec's TurboQuant path could improve our
TurboQuant implementation.

Our TurboQuant is already present across multiple access methods, including
HNSW, DiskANN, IVF, and SPIRE. Any useful change must therefore be evaluated as
a TurboQuant-family improvement that can either fit the shared quantizer/scorer
surface or be explicitly scoped to one AM before broader rollout.

## Scope

Investigate and, where justified, prototype TurboVec-derived TurboQuant changes
against our own TurboQuant baselines.

The first report and design packet must answer:

- what TurboVec means by encoding the query in the same space;
- whether TurboVec uses a new vector encoding, query transform, calibration, or
  scorer layout that makes query-time comparisons faster;
- how TurboVec's per-vector stored bytes compare to our TurboQuant bytes for
  the same dimensions and bit budgets;
- how TurboVec's SIMD/block kernels differ from our current TurboQuant SIMD
  kernels;
- what index type TurboVec implements, and how that affects transferability to
  HNSW, DiskANN, IVF, and SPIRE.

## Initial Findings To Validate

The starting hypothesis from local source inspection is:

- TurboVec is a flat exhaustive compressed-vector scanner, not HNSW, DiskANN,
  IVF, or a graph/partitioned AM.
- TurboVec does not appear to pack the query into the same per-vector database
  code format. It rotates the query into the same transformed coordinate space,
  applies TurboQuant+ inverse calibration, builds per-query lookup tables, and
  scores packed database codes directly.
- The likely transferable ideas are query-side calibration, per-coordinate
  shift/scale metadata, per-vector length renormalization, code-block layout,
  lower-precision lookup tables, fused scoring/top-k, and multi-query fused
  scoring.

These findings must be verified against the current checked-out TurboVec code
before any code slice lands.

## Candidate Workstreams

### Track A - TurboVec TurboQuant Report

Create a packet under `reviews/task-86/001-turbovec-tq-analysis/` with a
source-grounded report comparing TurboVec TurboQuant to our TurboQuant only.

The report must cite local source paths and cover:

- database-vector training and encoding path;
- query preparation path;
- scoring/LUT path;
- stored-code layout and per-vector byte accounting;
- SIMD/blocking strategy;
- index shape and implications for AM transfer.

Do not include RabitQ, grouped PQ, PqFastScan, or FAISS as comparison rows
except where needed to quote and dismiss TurboVec README claims as non-goals.

### Track B - TQ+ Calibration Prototype

Evaluate TurboVec's per-coordinate calibration idea for our TurboQuant:

- persisted or derived `scale` and `shift` metadata per rotated coordinate;
- query-side inverse calibration and bias handling;
- compatibility with current query preparation and AM scan paths;
- recall impact at unchanged code bytes and unchanged candidate surface.

The first prototype should be behind a narrow internal switch or benchmark-only
path. Do not change on-disk format without an explicit ADR or format-version
plan.

### Track C - Stored-Byte and Renormalization Semantics

Compare TurboVec's code budget and renormalization scalar against our current
TurboQuant representation:

- full scalar TQ code budget versus our active MSE/QJL split where applicable;
- per-vector length scalar semantics;
- whether TurboVec-style renormalization improves score error without adding
  hot bytes;
- whether any change can be represented as build-time metadata only.

This track must keep recall, latency, and bytes in the same packet so that a
quality win is not mistaken for a storage or scan regression.

### Track D - Blocked LUT and SIMD Scoring

Prototype only the scoring changes that plausibly transfer to our AMs:

- 32-vector or similar blocked code layout for contiguous scans;
- `u8` or otherwise narrower per-query lookup tables with explicit scale/bias;
- fused scoring plus top-k for flat or IVF-like candidate batches;
- multi-query fused scoring if the AM or benchmark runner can feed it without a
  broad API change.

This track must compare against our current TurboQuant SIMD kernels and scalar
fallbacks. It should start in the narrowest contiguous-candidate lane before
touching graph traversal.

## Non-Goals

- Do not compare TurboQuant to RabitQ, grouped PQ, PqFastScan, or any other
  quantizer as the point of this task.
- Do not produce a generic leaderboard or external comparison metric. Use our
  own TurboQuant baseline rows to decide whether a TurboVec-derived idea helps.
- Do not claim query-time decompression is eliminated unless the scorer proves
  database vectors remain packed and only per-query transforms/LUTs are built.
- Do not land a change that only works for flat exhaustive search unless the
  task explicitly records it as non-transferable to HNSW, DiskANN, IVF, and
  SPIRE.
- Do not write ad hoc benchmark sweep scripts. Use `ecaz bench suite` and
  extend the runner if a needed field is missing.

## Required Evidence

Every measurement packet must be packet-local under `reviews/task-86/` and use
`ecaz bench suite` for benchmark matrices.

The baseline packet must include:

- head SHA and TurboVec source snapshot or local path inspected;
- storage format and AM under test;
- dimensions, bit budget, code bytes, sidecar bytes, and metadata bytes;
- recall@10 or the task-appropriate recall metric;
- p50/p95/p99 latency;
- scorer timing where available;
- query preparation timing where available;
- stored bytes per vector and total index bytes;
- scalar and SIMD kernel variant used.

At least one evidence row should use a contiguous-candidate lane such as IVF or
a quantizer microbench. Broader AM rows for HNSW, DiskANN, and SPIRE should be
added only after a candidate idea shows promise in the narrow lane.

## Gates

A candidate improvement can proceed past prototype only if it shows one of:

- lower TurboQuant score error or higher recall at unchanged stored bytes and no
  material latency regression;
- lower query-time latency at unchanged recall and unchanged stored bytes;
- lower stored bytes at matched recall and no material latency regression;
- a clearly isolated AM-specific win with an explicit explanation of why it
  does or does not transfer to the other AMs.

The first implementation slice should not change durable on-disk format. Any
format-changing slice requires an ADR or task-local format-version plan before
landing.

## Implementation Order

1. Land the source-grounded TurboVec TurboQuant analysis packet.
2. Pick one low-risk prototype, preferably query-side calibration or narrower
   LUT scoring, and implement it behind a benchmark/internal switch.
3. Measure against our own TurboQuant baseline in the narrowest useful lane.
4. If the prototype passes, decide whether it belongs in shared TurboQuant,
   one AM-specific scorer, or a follow-up format task.
5. Only then broaden to HNSW, DiskANN, IVF, and SPIRE surfaces.

## Exit Criteria

- The TurboVec TurboQuant analysis report is committed under `reviews/task-86/`.
- At least one candidate improvement is either prototyped and measured or
  explicitly shelved with source-grounded reasoning.
- Any accepted code change has packet-backed TurboQuant baseline evidence.
- Any rejected idea explains whether the blocker is quality, latency, bytes,
  implementation complexity, or AM transferability.
- No unrelated quantizer work is included.
- No new unsafe blocks.
- PG18-focused validation is recorded for any code slice that changes scan,
  storage, or SQL-visible behavior.
