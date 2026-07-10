---
id: NFR-021
title: Distann Relay Resource and Depth Bounds
type: NFR
status: PROPOSED
quality_attribute: performance_efficiency
relationships:
  - target: "ix://agent-ix/ecaz/FR-087"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-088"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-089"
    type: "constrains"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/NFR-019"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/StR-008"
    type: "constrains"
    cardinality: "N:1"
---
# NFR-021: Distann Relay Resource and Depth Bounds

## Statement

BatANN-mode scans SHALL have bounded, stated resource occupancy: a stack-mode
scan of relay depth d SHALL occupy at most d+1 backends and d pooled
connections simultaneously (worst case `relay_max_depth + 1` backends per
query; direct-lite forwarding has the same bound); send-and-abandon
forwarding SHALL cap abandoned in-flight sends per backend and never leave a
pooled connection undrained after scan end; the relay-state size SHALL stay
within a documented envelope — beam ≤ seeds + expansions×R entries × 13 B,
hits ≤ expansions × 12 B, where R is the FR-076 `graph_degree` reloption —
which at the shipped defaults (BW=4, H=100, R=32,
budget fully spent) is ≈ 166 KB worst case, and typically far smaller under
convergence early-exit; the mailbox inline payload cap SHALL be sized
against this computed envelope (not the paper's 4–8 KB scale), with
oversize → delivered error status (FR-088-AC-6); and the direct-mode
mailbox SHALL be a fixed shared-memory budget (fixed slot array × inline
payload cap) allocated at startup. Cancel, timeout, and error paths SHALL
leave no orphaned backends, undrained connections, or leaked mailbox slots
(direct-mode residual drains bounded by the expansion and depth budgets are
not orphans provided they quiesce, ADR-086 D10). The NFR-019 BW×H expansion
bound holds unchanged in every coordination mode (the expansion budget
travels in the state and is the authoritative bound, ADR-086 D8/FR-085).

## Scope

- Applies to `batann_stack` and `batann_direct` scans at any roster size,
  and to the coordinator-mode resume after depth exhaustion (FR-089).
- Sizing guidance is normative documentation: worst-case backend consumption
  = concurrent_relay_queries × (relay_max_depth + 1), to be stated in the
  roster/transport operations documentation (the NFR-014 posture as lifted
  for distann by FR-079/FR-081). At the D6 default depth of min(H, 16) that
  is 17 backends per query worst case; operators raising `relay_max_depth`
  toward H (default 100) MUST size `max_connections` accordingly — the
  spec deliberately does not default to that regime.
- Stack-mode hang bound: `statement_timeout` on the coordinator (and
  intermediates) is the operator wait control (FR-087); drills cover
  cancel/timeout/error, and silent hangs are bounded by that timeout.

## Rationale

State passing trades the coordinator's per-hop RTT for occupancy of remote
backends and connections. That trade is only acceptable if the occupancy is
bounded and observable: exhaustion must present as classified connect/timeout
errors (FR-079 classes), never deadlock — no relay call ever waits on its own
backend (A→B→A lands in a fresh backend on A) — and resource leaks after
faults would poison a long-lived multinode deployment.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| simultaneous backends per stack-mode query | ≤ depth+1 | never exceeded | fixture drill: `debug_hold_relay_depth` stalls the chain at max depth, then count `application_name = 'ec_distann_relay'` backends in pg_stat_activity **per instance, across all roster nodes** |
| undrained pooled connections / orphaned relay backends after cancel, timeout, error drills | 0 | 0 | TC-046/TC-047 drill assertions with an explicit settle rule: poll pg_stat_activity per instance until quiesce, bounded |
| leaked mailbox slots after success/timeout/abort drills | 0 | 0 | `ec_distann_relay_mailbox_status()` after TC-047 drills |
| encoded relay-state bytes at default BW/H/R (fixture) | within documented envelope | ≤ envelope | `state_bytes_max` counter assertion |
| expansions per attempt across all drains + resume | ≤ BW×H | never exceeded | NFR-019 counter assertion re-run in each mode |

## Verification

The TC-046/TC-047 drill matrices assert the zero-leak rows; the bench
pipeline step re-asserts the NFR-019 cap and the state-size envelope per
cell in every coordination mode. Any breach fails the run.

## Dependencies

- **Upstream**: [FR-087](../functional/index/distann/FR-087-distann-stack-return.md),
  [FR-088](../functional/index/distann/FR-088-distann-direct-return.md),
  [FR-089](../functional/index/distann/FR-089-distann-relay-depth-hybrid-resume.md),
  [NFR-019](./NFR-019-distann-per-query-touch-bound.md),
  [StR-008](../stakeholder/StR-008-distributed-search-single-instance-economics.md)
