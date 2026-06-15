# Task 107 Packet 004 Run Checklist

Date: 2026-06-15
Head SHA: `97d8c85cf62ac7ade1de2923799e975dad6e6818`
Packet: `reviews/task-107/004-distributed-completion/`

## Current State

- AWS topology: one coordinator plus two remotes.
- Current AWS state: running after completing `phase1-rabitq-100k-l4`;
  proceed one cell at a time.
- Packet 004 benchmark/load status: completed
  `phase1-rabitq-100k-l1-control`, `phase1-rabitq-100k-l2`, and
  `phase1-rabitq-100k-l4` with
  packet-local load/build, storage, recall/latency, and cleanup evidence.
- Existing completed Task 107 benchmark evidence:
  - Packet 003: RaBitQ 100k distributed, `bits=4`, `local_store_count=1`.
  - Packet 004: RaBitQ 100k single-node/multi-store control,
    `bits=4`, `local_store_count=1`.
  - Packet 004: RaBitQ 100k single-node/multi-store,
    `bits=4`, `local_store_count=2`, explicit
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`.
  - Packet 004: the earlier
    `artifacts/phase1-rabitq-100k-l2/direct-ssm/` run is superseded because it
    omitted explicit `local_store_tablespaces`.
  - Packet 004: RaBitQ 100k single-node/multi-store,
    `bits=4`, `local_store_count=4`, explicit
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2,ecaz_spire_store_3,ecaz_spire_store_4`.

## Operating Rules

1. Run one benchmark cell at a time. Do not build multiple indexes at once.
2. Do not rerun Task 106 single-store SPIRE evidence.
3. Do not rerun HNSW, IVF, DiskANN, or other comparator baselines.
4. Use `ecaz bench suite` for benchmark matrices and sweeps.
5. Do not patch infrastructure or runner scripts unless the next checklist cell
   is blocked by a concrete failure. Record the failure before any patch.
6. Before starting each cell, record the exact command and intended artifact
   directory.
7. After each cell, record status, elapsed time, key result lines, cleanup
   state, and current AWS state. Leave the Task 107 instances running unless
   the user explicitly asks to stop them or a concrete failure requires cleanup.
8. A cell is complete only when packet-local artifacts include load/build,
   storage, recall/latency, and required routing/fanout evidence.
9. Do not impose arbitrary wall-clock caps on benchmark cells. Long AWS SSM
   cells must set both the send-command timeout and the `AWS-RunShellScript`
   `executionTimeout` high enough to let the cell run to completion or an
   actual command failure.

## Required Cells

### Phase 1 - Single-Node Multi-Disk / Multi-Store

These are Task 107 cells and were not completed by Task 106. They should run on
the Task 107 AWS host with the multi-store device layout, one index lane at a
time.

| Cell | Scale | Storage | Store count | Status | Artifact directory |
| --- | --- | --- | ---: | --- | --- |
| phase1-rabitq-100k-l1-control | 100k | RaBitQ | 1 | Completed in packet 004 direct SSM retry; cleanup completed | `artifacts/phase1-rabitq-100k-l1-control/retry-direct-ssm/` |
| phase1-rabitq-100k-l2 | 100k | RaBitQ | 2 | Completed in packet 004 corrected direct SSM tablespace run; cleanup completed | `artifacts/phase1-rabitq-100k-l2/direct-ssm-tablespaces/` |
| phase1-rabitq-100k-l4 | 100k | RaBitQ | 4 | Completed in packet 004 corrected direct SSM tablespace run; cleanup completed | `artifacts/phase1-rabitq-100k-l4/direct-ssm-tablespaces/` |
| phase1-rabitq-1m-l1-control | 1m | RaBitQ | 1 | Not started in packet 004 | `artifacts/phase1-rabitq-1m-l1-control/` |
| phase1-rabitq-1m-l2 | 1m | RaBitQ | 2 | Not started | `artifacts/phase1-rabitq-1m-l2/` |
| phase1-rabitq-1m-l4 | 1m | RaBitQ | 4 | Not started | `artifacts/phase1-rabitq-1m-l4/` |
| phase1-turboquant-100k-l1-control | 100k | TurboQuant | 1 | Not started in packet 004 | `artifacts/phase1-turboquant-100k-l1-control/` |
| phase1-turboquant-100k-l2 | 100k | TurboQuant | 2 | Not started | `artifacts/phase1-turboquant-100k-l2/` |
| phase1-turboquant-100k-l4 | 100k | TurboQuant | 4 | Not started | `artifacts/phase1-turboquant-100k-l4/` |
| phase1-turboquant-1m-l1-control | 1m | TurboQuant | 1 | Not started | `artifacts/phase1-turboquant-1m-l1-control/` |
| phase1-turboquant-1m-l2 | 1m | TurboQuant | 2 | Not started | `artifacts/phase1-turboquant-1m-l2/` |
| phase1-turboquant-1m-l4 | 1m | TurboQuant | 4 | Not started | `artifacts/phase1-turboquant-1m-l4/` |

### Phase 2 - Distributed Multi-Node

These cells run on the one-coordinator/two-remote topology.

| Cell | Scale | Storage | Store count | Status | Artifact directory |
| --- | --- | --- | ---: | --- | --- |
| phase2-rabitq-100k-l1 | 100k | RaBitQ | 1 | Completed in packet 003; cite, do not rerun unless invalidated | `../003-aws-benchmarks/artifacts/rabitq-100k-l1/` |
| phase2-rabitq-1m-l1 | 1m | RaBitQ | 1 | Packet 003 attempt canceled during coordinator index build; no completed run | `artifacts/phase2-rabitq-1m-l1/` |
| phase2-turboquant-100k-l1 | 100k | TurboQuant | 1 | Packet 003 has coordinator load/build only; distributed result missing | `artifacts/phase2-turboquant-100k-l1/` |
| phase2-turboquant-1m-l1 | 1m | TurboQuant | 1 | Not started | `artifacts/phase2-turboquant-1m-l1/` |

## Stop/Go Checkpoints

- Before any AWS start: confirm the next cell and intended artifact directory.
- Before any 1m build: confirm prepared corpus prefix and scale; do not use the
  default 100k representative prefix by accident.
- Before TurboQuant distributed full runs: first record the current endpoint
  readiness behavior. If the distributed remote read path reports TurboQuant as
  unsupported, pause benchmark sequencing and package that blocker instead of
  running 1m TurboQuant.
- After any failed cell: pause benchmark sequencing, record the exact failure,
  and ask whether to fix the blocker or move to the next independent cell. Keep
  the Task 107 EC2 instances running unless the user explicitly asks otherwise
  or a concrete cleanup requirement makes that unsafe.
- After each work session: record current EC2 state. Leave AWS running unless
  the user explicitly asks to stop it.
