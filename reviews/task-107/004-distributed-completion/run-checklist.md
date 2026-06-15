# Task 107 Packet 004 Run Checklist

Date: 2026-06-15
Head SHA: `92254bca929c949f2de3715efefec6d4c53e4568`
Packet: `reviews/task-107/004-distributed-completion/`

## Current State

- AWS topology: one coordinator plus two remotes.
- Current AWS state: running after completing all checklist benchmark cells;
  Task 107 benchmark DB objects have been cleaned up after the final cell.
- Packet 004 benchmark/load status for Task 107 Phase 1 cells: completed
  `phase1-rabitq-100k-l2`, `phase1-rabitq-1m-l2`,
  `phase1-turboquant-100k-l2`, and `phase1-turboquant-1m-l2` with
  packet-local load/build, storage, recall/latency, and cleanup evidence.
- Packet 004 Phase 2 TurboQuant distributed status: completed after commit
  `92254bca929c949f2de3715efefec6d4c53e4568` enabled TurboQuant SPIRE remote
  endpoints. `phase2-turboquant-100k-l1` completed successfully.
  `phase2-turboquant-1m-l1` completed all suite steps and cleanup, with a
  recorded suite threshold miss: k10 nprobe64 recall measured `0.9490` against
  the configured `>= 0.9500` floor, causing two threshold checks to fail.
- Existing completed Task 107 benchmark evidence:
  - Packet 003: RaBitQ 100k distributed, `bits=4`, `local_store_count=1`.
  - Packet 004: RaBitQ 1m distributed, `bits=4`, `local_store_count=1`,
    one coordinator plus two remotes.
  - Packet 004: RaBitQ 100k single node with 2 disks,
    `bits=4`, `local_store_count=2`, explicit
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`.
  - Packet 004: TurboQuant 100k single node with 2 disks,
    `bits=4`, `local_store_count=2`, explicit
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`.
  - Packet 004: TurboQuant 1m single node with 2 disks,
    `bits=4`, `local_store_count=2`, explicit
    `local_store_tablespaces=ecaz_spire_store_1,ecaz_spire_store_2`.
  - Packet 004: TurboQuant 100k distributed, `bits=4`,
    `local_store_count=1`, one coordinator plus two remotes, completed after
    the remote endpoint fix.
  - Packet 004: TurboQuant 1m distributed, `bits=4`,
    `local_store_count=1`, one coordinator plus two remotes, completed with
    the threshold miss recorded above.
  - Packet 004: the earlier
    `artifacts/phase1-rabitq-100k-l2/direct-ssm/` run is superseded because it
    omitted explicit `local_store_tablespaces`.

## Operating Rules

1. The checklist contains only these test classes:
   - single node with 2 disks;
   - multinode with 1 controller and 2 nodes.
2. Remove any test that does not match those classes from this checklist.
3. Run one benchmark cell at a time. Do not build multiple indexes at once.
4. Do not rerun Task 106 single-node/single-disk SPIRE evidence.
5. Do not add or run Phase 1 single-node/single-disk rows. They are outside
   this checklist.
6. Do not add or run Phase 1 4-disk rows. They are outside the requested scope.
7. Do not rerun HNSW, IVF, DiskANN, or other comparator rows.
8. Use `ecaz bench suite` for benchmark matrices and sweeps.
9. Do not patch infrastructure or runner scripts unless the next checklist cell
   is blocked by a concrete failure. Record the failure before any patch.
10. Before starting each runnable cell, record the exact command and intended artifact
   directory.
11. After each runnable cell, record status, elapsed time, key result lines, cleanup
   state, and current AWS state. Leave the Task 107 instances running unless
   the user explicitly asks to stop them or a concrete failure requires cleanup.
12. A runnable cell is complete only when packet-local artifacts include load/build,
   storage, recall/latency, and required routing/fanout evidence.
13. Do not impose arbitrary wall-clock caps on benchmark cells. Long AWS SSM
   cells must set both the send-command timeout and the `AWS-RunShellScript`
   `executionTimeout` high enough to let the cell run to completion or an
   actual command failure.

## Required Cells

### Phase 1 - Single Node With 2 Disks

Only 2-disk rows are runnable Task 107 single-node cells. Single-node/single-
disk rows and four-disk rows are outside this checklist. Runnable rows should
run on the Task 107 AWS host with the two-store device layout, one index lane
at a time.

| Cell | Scale | Storage | Store count | Status | Artifact directory |
| --- | --- | --- | ---: | --- | --- |
| phase1-rabitq-100k-l2 | 100k | RaBitQ | 2 | Completed in packet 004 corrected direct SSM tablespace run; cleanup completed | `artifacts/phase1-rabitq-100k-l2/direct-ssm-tablespaces/` |
| phase1-rabitq-1m-l2 | 1m | RaBitQ | 2 | Completed in packet 004 direct SSM tablespace run; cleanup completed | `artifacts/phase1-rabitq-1m-l2/direct-ssm-tablespaces/` |
| phase1-turboquant-100k-l2 | 100k | TurboQuant | 2 | Completed in packet 004 direct SSM tablespace run; cleanup completed | `artifacts/phase1-turboquant-100k-l2/direct-ssm-tablespaces/` |
| phase1-turboquant-1m-l2 | 1m | TurboQuant | 2 | Completed in packet 004 direct SSM tablespace run; cleanup completed | `artifacts/phase1-turboquant-1m-l2/direct-ssm-tablespaces/` |

### Phase 2 - Multinode With 1 Controller And 2 Nodes

These cells run on the one-coordinator/two-remote topology.

| Cell | Scale | Storage | Store count | Status | Artifact directory |
| --- | --- | --- | ---: | --- | --- |
| phase2-rabitq-100k-l1 | 100k | RaBitQ | 1 | Completed in packet 003; cite, do not rerun unless invalidated | `../003-aws-benchmarks/artifacts/rabitq-100k-l1/` |
| phase2-rabitq-1m-l1 | 1m | RaBitQ | 1 | Completed in packet 004 direct SSM distributed run; coordinator/remotes cleanup completed | `artifacts/phase2-rabitq-1m-l1/direct-ssm-distributed/` |
| phase2-turboquant-100k-l1 | 100k | TurboQuant | 1 | Completed in packet 004 after TurboQuant remote endpoint fix; coordinator/remotes cleanup completed | `artifacts/phase2-turboquant-100k-l1/direct-ssm-distributed/` |
| phase2-turboquant-1m-l1 | 1m | TurboQuant | 1 | Completed in packet 004 direct SSM distributed run; all suite steps succeeded, suite returned threshold failure because k10 nprobe64 recall was `0.9490` vs configured `>=0.9500`; coordinator/remotes cleanup completed | `artifacts/phase2-turboquant-1m-l1/direct-ssm-distributed/` |

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
