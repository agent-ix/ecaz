# SPIRE Paper Parity Seed Checklist

Reference basis:

- Paper cross-check from
  `review/30658-spire-phase9-routing-plan/feedback/2026-05-09-01-reviewer.md`
  against `/home/peter/dev_bak/papers/2512.17264v1.pdf`.
- Phase 9 and 10 local architecture closeout in:
  - `plan/tasks/task30-phase9-spire-graph-architecture.md`
  - `plan/tasks/task30-phase10-spire-execution-performance.md`

This seed is not acceptance evidence. It is the starting checklist for Phase
11.1.

## Parity Matrix

| Paper / Production Mechanism | Current State | Phase 11 Target |
| --- | --- | --- |
| Hierarchical root/top graph routing | Local architecture closed in Phase 9; top graph storage and global route budgets landed. | Keep as dependency; no semantic rewrite expected unless distributed execution exposes a gap. |
| Level-by-level bounded descent | Global route budget and diagnostics landed. | Use the same budget controls across distributed fanout and remote endpoints. |
| Boundary replication | Local execution contract and diagnostics landed; opt-in remains measured locally. | Prove end-to-end dedupe across remote nodes after writer-side global IDs. |
| Stable cross-node identity | ADR-055 defines global `0x02` IDs; remote merge scopes local IDs by node. | Emit writer-side global IDs and test cross-node replica dedupe end to end. |
| Disaggregated/stateless query execution | Diagnostic/operator libpq surfaces exist; ADR-058 keeps current executor diagnostic-only. | Add production coordinator executor with concurrent or pipelined remote fanout. |
| Remote near-data scoring | SQL contracts and diagnostic candidate rows exist. | Promote/add production remote endpoint with served epoch, compact candidates, and strict/degraded behavior. |
| Remote heap/final row delivery | ADR-059 assigns origin-node heap resolution; current production state is blocked. | Implement origin-node heap visibility filtering and one coordinator-visible ordered stream. |
| Multi-instance consistency | Epoch and remote diagnostics exist, but no production multi-instance fixture gate. | Add coordinator plus at least two remote PostgreSQL node fixture and strict/degraded tests. |
| Multi-NVMe/store execution | Local store grouping, prefetch/read-stream contract, and pipeline counters exist. | Harden local multi-store harness and diagnostics before AWS. |
| Production observability | Strong local SQL diagnostics and `ecaz bench spire-pipeline` exist. | Add distributed recall/latency/counter harness and production runbook. |
| Quantized scoring | RaBitQ is supported first; PQ/PQFastScan unsupported/reserved. | Keep RaBitQ scope; explicitly defer PQ/PQFastScan from paper-parity claims. |
| AWS/RDS scale evidence | Scale packet remains open. | Defer until Phase 11 local production-readiness bundle passes review. |

## First Slice Candidates

1. Phase 11.1 paper-parity gate packet:
   - Convert this seed into an accepted task-level checklist.
   - Mark diagnostic-only vs production surfaces in docs.
   - Define exact AWS entry criteria.
2. Phase 11.2 writer-side global ID implementation:
   - Highest correctness prerequisite for boundary replicas and distributed
     merge.
3. Phase 11.3/11.4 remote endpoint plus coordinator executor design split:
   - Decide whether to land the endpoint first, then pipeline coordinator, or
     land both behind a production-readiness gate.

Recommended first implementation slice: writer-side global vector IDs, because
it is a hard correctness dependency for any production remote merge claim.
