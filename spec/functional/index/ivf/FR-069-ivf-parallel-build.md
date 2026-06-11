---
id: FR-069
title: "IVF Parallel Build Process"
artifact_type: FR
status: IMPLEMENTED
object: process
relationships:
  - target: "ix://agent-ix/ecaz/US-013"
    type: "implements"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-031"
    type: "references"
    cardinality: "1:1"
  - target: "ix://agent-ix/ecaz/FR-061"
    type: "writes"
    cardinality: "N:1"
---
# [FR-069] IVF Parallel Build Process

## Description

This process object defines the leader/worker coordination used by parallel
`ec_ivf` builds. `FR-031` requires serial/parallel equivalence for centroid
assignment and posting-list metadata before parallel builds support
product-scale build-time claims; this FR pins the coordination protocol that
equivalence proof runs against.

Implementation anchor: `src/am/ec_ivf/build_parallel.rs`
(`EcIvfParallelBuildPlan`, `EcIvfParallelBuildSharedHeader`,
`ec_ivf_parallel_build_main`, `ec_ivf_parallel_build_callback`).

## Workflow

```mermaid
sequenceDiagram
    participant PG as PostgreSQL (ii_ParallelWorkers)
    participant Leader as ambuild leader
    participant DSM as DSM shared state (PARALLEL_KEY_EC_IVF_BUILD_SHARED = 0xEC1F000000000001)
    participant W as Parallel workers (ec_ivf_parallel_build_main)
    participant Idx as Index pages (FR-061)

    PG->>Leader: ambuild with parallel workers from IndexInfo
    Leader->>DSM: initialize EcIvfParallelBuildSharedHeader (plan, snapshot, message queues)
    Leader->>W: launch workers
    par each worker
        W->>W: parallel heap scan chunk; capture tuples
        loop per captured tuple
            W->>Leader: BUILD_TUPLE_MESSAGE (= 1) with tuple payload
        end
        W->>Leader: BUILD_DONE_MESSAGE (= 2)
    end
    Leader->>Leader: train centroids deterministically (data + seed)
    Leader->>Leader: assign captured tuples to posting lists
    Leader->>Idx: write metadata, centroid chain, directory, posting pages
    Leader-->>PG: build result (workers_launched, parallel_effective_workers in build diagnostics)
```

## Specification

- PostgreSQL remains the worker-count authority: the plan consumes
  `ii_ParallelWorkers`; there is no ecaz-side worker GUC for IVF build.
- Workers only capture and forward heap tuples; centroid training and
  posting-list assignment happen on the leader so the persisted result is a
  pure function of (heap data, seed), independent of worker interleaving.
- The message protocol is two-valued: `BUILD_TUPLE_MESSAGE = 1` carries one
  captured tuple; `BUILD_DONE_MESSAGE = 2` closes a worker's stream.
- Build diagnostics expose `workers_launched` and
  `parallel_effective_workers`, which `ecaz bench suite` parses into
  build-timing result rows (`FR-066`).

## Algorithm

1. Leader plans the parallel build and publishes shared state under the
   reserved DSM key.
2. Workers run disjoint parallel heap-scan chunks and stream captured tuples
   to the leader queue.
3. After every worker signals done, the leader trains centroids with the
   deterministic `(data, seed)` procedure from `FR-031` behavior 5.
4. The leader assigns all tuples to lists and writes the `FR-061` format
   exactly as the serial path would.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-069-CON-1 | Serial and parallel builds over the same heap data and seed produce equivalent index-visible results (centroid assignment and posting metadata) | Technical | pg_test fixture comparison (Task 71 fixtures, `FR-031-AC-4`) |
| FR-069-CON-2 | Worker tuple capture is provably live in diagnostics (no silent serial fallback) | Technical | Build diagnostics assertion |
| FR-069-CON-3 | Parallel build-time claims in packets cite `workers_launched`/`parallel_effective_workers` evidence from the suite manifest or build-timing rows | Business | Packet review (`NFR-007`) |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-069-AC-1 | A parallel build on the validation fixtures matches the serial build's index-visible results | pg_test |
| FR-069-AC-2 | Build diagnostics distinguish requested, launched, and effective worker counts | pg_test |
| FR-069-AC-3 | Suite build-timing rows for an IVF parallel build include worker fields | CLI unit test |

## Dependencies

- **Upstream**: PostgreSQL parallel-build infrastructure, `FR-021` parallel-build groundwork.
- **Downstream**: `FR-031` behavioral requirements, `FR-061` persisted format, `FR-066` build-timing rows.
